/**
 * State and Next handler of upstream `RemoteAccessStep.qml`: the SSH
 * switch, password vs public-key authentication, the key list, and the
 * runtime + saved settings (`sshEnabled`, `sshPasswordAuth`,
 * `sshAuthorizedKeys`, legacy `sshPublicKey` removed).
 */
import { useState } from 'react'

import { call } from '@/api/imager'

import type { CustomizationSettings } from '../../state'
import { useWizard, type WizardApi } from '../../WizardContext'
import { editSettings, isTrue, rewriteSaved } from './settings'
import { savedKeys } from './sshKeys'
import { useMountEffect } from './useMountEffect'

export interface RemoteAccessFields {
  enabled: boolean
  publicKey: boolean
  keys: readonly string[]
}

const SSH_KEYS = ['sshEnabled', 'sshPasswordAuth', 'sshAuthorizedKeys', 'sshPublicKey'] as const

export function initialRemoteAccess(settings: CustomizationSettings): RemoteAccessFields {
  const keys = savedKeys(settings)
  return { enabled: isTrue(settings.sshEnabled), publicKey: keys.length > 0, keys }
}

/** Upstream `nextButtonEnabled`. */
export function remoteNextEnabled(f: RemoteAccessFields): boolean {
  return !f.enabled || !f.publicKey || f.keys.length > 0
}

/** The settings change for `f`: what to set and what to delete. */
export function remoteChange(f: RemoteAccessFields): {
  patch: CustomizationSettings
  remove: string[]
} {
  if (!f.enabled) return { patch: {}, remove: [...SSH_KEYS] }
  const withKeys = f.publicKey && f.keys.length > 0
  const patch: Record<string, unknown> = { sshEnabled: true, sshPasswordAuth: !f.publicKey }
  if (withKeys) patch.sshAuthorizedKeys = f.keys.join('\n')
  return {
    patch,
    remove: withKeys ? ['sshPublicKey'] : ['sshAuthorizedKeys', 'sshPublicKey'],
  }
}

/** Upstream `onNextClicked`. */
export async function saveRemoteAccess(wizard: WizardApi, f: RemoteAccessFields): Promise<void> {
  const { patch, remove } = remoteChange(f)
  editSettings(wizard, patch, remove)
  wizard.update({ sshEnabled: f.enabled })
  await rewriteSaved(patch, remove)
}

export function useRemoteAccess() {
  const wizard = useWizard()
  const [fields, setFields] = useState(() => initialRemoteAccess(wizard.customizationSettings))
  const [embedded, setEmbedded] = useState(true)
  const [busy, setBusy] = useState(false)
  useMountEffect(() => {
    void call<boolean | null>('isEmbeddedMode').then((value) => setEmbedded(Boolean(value)))
  })
  const set = (patch: Partial<RemoteAccessFields>) => setFields((f) => ({ ...f, ...patch }))
  const submit = async () => {
    setBusy(true)
    try {
      await saveRemoteAccess(wizard, fields)
      wizard.next()
    } finally {
      setBusy(false)
    }
  }
  return { fields, set, embedded, submit, nextEnabled: !busy && remoteNextEnabled(fields) }
}
