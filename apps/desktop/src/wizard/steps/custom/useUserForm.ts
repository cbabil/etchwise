/**
 * State and rules of upstream `UserCustomizationStep.qml`: the username
 * (input-restricted), password and confirmation, the "keep the saved
 * hash" semantics, the session-only passwordless sudo flag, and the Next
 * handler that hashes the password with `crypt` (never storing plaintext).
 */
import { useState } from 'react'

import { call } from '@/api/imager'

import { useWizard, type WizardApi } from '../../WizardContext'
import { editSettings, persist, str, unpersist } from './settings'
import { acceptsInput, USERNAME_RE } from './validation'

export interface UserFields {
  username: string
  password: string
  confirm: string
  passwordlessSudo: boolean
}

/** Upstream `nextButtonEnabled`. */
export function userNextEnabled(f: UserFields, hasSaved: boolean): boolean {
  const allEmpty = f.username.length === 0 && f.password.length === 0 && f.confirm.length === 0
  const newMatch = f.password.length > 0 && f.confirm.length > 0 && f.password === f.confirm
  const keepSaved = hasSaved && f.password.length === 0 && f.confirm.length === 0
  return allEmpty || newMatch || keepSaved
}

const sudoPatch = (wizard: WizardApi, on: boolean) =>
  on
    ? editSettings(wizard, { passwordlessSudo: true })
    : editSettings(wizard, {}, ['passwordlessSudo'])

/** Upstream `onNextClicked`. */
export async function saveUser(wizard: WizardApi, f: UserFields, hasSaved: boolean): Promise<void> {
  const username = f.username.trim()
  const hasPasswords = f.password.length > 0 && f.password === f.confirm
  if (username.length > 0 && hasPasswords) {
    const crypted = String((await call<string | null>('crypt', f.password)) ?? '')
    editSettings(wizard, { sshUserName: username, sshUserPassword: crypted })
    wizard.update({ userConfigured: true })
    persist('sshUserName', username)
    persist('sshUserPassword', crypted)
    sudoPatch(wizard, f.passwordlessSudo)
  } else if (username.length > 0 && hasSaved && f.password.length === 0) {
    editSettings(wizard, { sshUserName: username })
    wizard.update({ userConfigured: true })
    persist('sshUserName', username)
    sudoPatch(wizard, f.passwordlessSudo)
  } else if (username.length === 0 && f.password.length === 0 && f.confirm.length === 0) {
    editSettings(wizard, {}, ['sshUserName', 'sshUserPassword', 'passwordlessSudo'])
    wizard.update({ userConfigured: false })
    unpersist('sshUserName')
    unpersist('sshUserPassword')
  } else {
    wizard.update({ userConfigured: false })
  }
}

export function useUserForm() {
  const wizard = useWizard()
  const [hasSaved] = useState(() => Boolean(wizard.customizationSettings.sshUserPassword))
  const [fields, setFields] = useState<UserFields>(() => ({
    username: str(wizard.customizationSettings.sshUserName),
    password: '',
    confirm: '',
    // Never restored: must be enabled explicitly every session.
    passwordlessSudo: false,
  }))
  const [busy, setBusy] = useState(false)
  const set = (patch: Partial<UserFields>) => setFields((f) => ({ ...f, ...patch }))
  const setUsername = (value: string) =>
    acceptsInput(USERNAME_RE, value) && set({ username: value })
  const submit = async () => {
    setBusy(true)
    try {
      await saveUser(wizard, fields, hasSaved)
      wizard.next()
    } finally {
      setBusy(false)
    }
  }
  const nextEnabled = !busy && userNextEnabled(fields, hasSaved)
  return { fields, hasSaved, set, setUsername, submit, nextEnabled }
}
