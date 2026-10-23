/** State and actions of upstream `SecureBootCustomizationStep.qml`. */
import { useEffect, useState } from 'react'

import { call } from '@/api/imager'
import { useTr } from '@/i18n'

import { pickFile } from '../../dialogs/common/pickFile'
import { useWizard } from '../../WizardContext'
import { isTrue } from './layout'

export const RSA_KEY_SETTING = 'secureboot_rsa_key'

/** Upstream persists the Secure Boot choice in the saved customisation. */
async function persistEnabled(enabled: boolean): Promise<void> {
  const saved = (await call<Record<string, unknown> | null>('getSavedCustomisationSettings')) ?? {}
  const rest = Object.fromEntries(Object.entries(saved).filter(([k]) => k !== 'secureBootEnabled'))
  await call('setSavedCustomisationSettings', enabled ? { ...rest, secureBootEnabled: true } : rest)
}

function useKeyState() {
  const [keyPath, setKeyPath] = useState('')
  const [fingerprint, setFingerprint] = useState('')
  const [rpiboot, setRpiboot] = useState(false)
  useEffect(() => {
    let live = true
    void call<string | null>('getStringSetting', RSA_KEY_SETTING).then((path) => {
      if (live) setKeyPath(path ?? '')
    })
    void call<boolean | null>('isRpibootDevice').then((value) => {
      if (live) setRpiboot(value === true)
    })
    return () => {
      live = false
    }
  }, [])
  useEffect(() => {
    if (!keyPath) return
    let live = true
    void call<string | null>('getRsaKeyFingerprint', keyPath).then((value) => {
      if (live) setFingerprint(value ?? '')
    })
    return () => {
      live = false
    }
  }, [keyPath])
  return { keyPath, setKeyPath, fingerprint: keyPath ? fingerprint : '', rpiboot }
}

export function useSecureBoot() {
  const tr = useTr('SecureBootCustomizationStep')
  const wizard = useWizard()
  const key = useKeyState()
  const [checked, setChecked] = useState(() =>
    isTrue(wizard.customizationSettings.secureBootEnabled),
  )
  const selectKey = async () => {
    const path = await pickFile({
      title: tr('Select RSA Private Key'),
      filters: [
        { name: tr('PEM Files (*.pem)'), extensions: ['pem'] },
        { name: tr('All Files (*)'), extensions: ['*'] },
      ],
      homeSubdir: '.ssh',
    })
    if (!path) return
    void call('setSetting', RSA_KEY_SETTING, path)
    key.setKeyPath(path)
    wizard.update({ secureBootKeyConfigured: true })
  }
  const toggle = (value: boolean) => {
    setChecked(value)
    wizard.update({ secureBootEnabled: value })
  }
  const next = () => {
    if (checked) wizard.updateCustomizationSettings({ secureBootEnabled: true })
    else wizard.removeCustomizationSetting('secureBootEnabled')
    wizard.update({ secureBootEnabled: checked })
    void persistEnabled(checked)
    wizard.next()
  }
  return { ...key, checked, toggle, selectKey, next }
}
