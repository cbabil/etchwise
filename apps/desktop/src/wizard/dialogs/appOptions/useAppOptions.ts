/** Load-on-open and Save of upstream `AppOptionsDialog.qml`. */
import { useEffect, useState } from 'react'

import { call } from '@/api/imager'

import { useWizard } from '../../WizardContext'
import { isWriteIdle } from '../common/writeIdle'

export interface AppOptionsValues {
  beep: boolean
  eject: boolean
  disableWarnings: boolean
  rsaKeyPath: string
}

export interface AppOptionsEnv {
  beepAvailable: boolean
  idle: boolean
  secureBootVisible: boolean
  version: string
}

const emptyValues: AppOptionsValues = {
  beep: false,
  eject: false,
  disableWarnings: false,
  rsaKeyPath: '',
}

const defaultEnv: AppOptionsEnv = {
  beepAvailable: false,
  idle: true,
  secureBootVisible: false,
  version: '',
}

async function loadOptions(secureBootAvailable: boolean, disableWarnings: boolean) {
  const flag = (method: string, ...args: unknown[]) =>
    call<boolean | null>(method, ...args).then(Boolean)
  const [beepAvailable, beep, eject, rsaKeyPath, idle] = await Promise.all([
    flag('isBeepAvailable'),
    flag('getBoolSetting', 'beep'),
    flag('getBoolSetting', 'eject'),
    call<string | null>('getStringSetting', 'secureboot_rsa_key'),
    isWriteIdle(),
  ])
  const [cliSecureBoot, swSecureBoot, version] = await Promise.all([
    flag('isSecureBootForcedByCliFlag'),
    flag('checkSWCapability', 'secure_boot'),
    call<string | null>('constantVersion'),
  ])
  const values = {
    beep: beep && beepAvailable,
    eject,
    disableWarnings,
    rsaKeyPath: rsaKeyPath ?? '',
  }
  const env: AppOptionsEnv = {
    beepAvailable,
    idle,
    secureBootVisible: secureBootAvailable || cliSecureBoot || swSecureBoot,
    version: version ?? '',
  }
  return { values, env }
}

export function useAppOptions(open: boolean) {
  const wizard = useWizard()
  const [values, setValues] = useState<AppOptionsValues>(emptyValues)
  const [env, setEnv] = useState<AppOptionsEnv>(defaultEnv)
  const { secureBootAvailable, disableWarnings } = wizard
  useEffect(() => {
    if (!open) return
    let live = true
    void loadOptions(secureBootAvailable, disableWarnings).then((loaded) => {
      if (!live) return
      setValues(loaded.values)
      setEnv(loaded.env)
    })
    return () => {
      live = false
    }
    // Reload only when the dialog opens, like upstream `onOpened`.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open])
  const change = (patch: Partial<AppOptionsValues>) => setValues((v) => ({ ...v, ...patch }))
  const save = () => {
    void call('setSetting', 'beep', values.beep && env.beepAvailable)
    void call('setSetting', 'eject', values.eject)
    void call('setSetting', 'secureboot_rsa_key', values.rsaKeyPath)
    wizard.update({ disableWarnings: values.disableWarnings })
  }
  return { values, env, change, save }
}
