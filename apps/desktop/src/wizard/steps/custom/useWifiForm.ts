/**
 * Field state of upstream `WifiCustomizationStep.qml`: SECURE / OPEN mode,
 * SSID, password + confirmation and the hidden-SSID flag, with the saved
 * SSID / crypt that decide whether a blank password keeps the saved hash.
 */
import { useState } from 'react'

import { useWizard } from '../../WizardContext'
import { isTrue, str } from './settings'
import {
  ssidUnchanged,
  type WifiMode,
  wifiNextEnabled,
  wifiPasswordError,
  type WifiPasswordState,
} from './validation'

export interface WifiFields {
  mode: WifiMode
  ssid: string
  password: string
  confirm: string
  hidden: boolean
}

export function initialWifiFields(settings: Readonly<Record<string, unknown>>): WifiFields {
  const saved = settings.wifiMode
  return {
    mode: saved === 'secure' || saved === 'open' ? saved : 'secure',
    ssid: str(settings.wifiSSID),
    password: '',
    confirm: '',
    hidden: isTrue(settings.wifiHidden),
  }
}

export function useWifiForm() {
  const wizard = useWizard()
  const [saved] = useState(() => ({
    ssid: str(wizard.customizationSettings.wifiSSID),
    hadCrypt: Boolean(wizard.customizationSettings.wifiPasswordCrypt),
  }))
  const [fields, setFields] = useState<WifiFields>(() =>
    initialWifiFields(wizard.customizationSettings),
  )
  const set = (patch: Partial<WifiFields>) => setFields((f) => ({ ...f, ...patch }))
  /** Upstream `updatePasswordFieldUI()`: open mode clears both password fields. */
  const setMode = (mode: WifiMode) =>
    set(mode === 'open' ? { mode, password: '', confirm: '' } : { mode })
  const canKeep = saved.hadCrypt && ssidUnchanged(fields.ssid.trim(), saved.ssid)
  const pw: WifiPasswordState = {
    mode: fields.mode,
    password: fields.password,
    confirm: fields.confirm,
    canKeep,
  }
  return {
    fields,
    set,
    setMode,
    hadSavedCrypt: saved.hadCrypt,
    canKeep,
    showPw: fields.mode === 'secure',
    error: wifiPasswordError(pw),
    nextEnabled: wifiNextEnabled(fields.ssid, pw),
  }
}

export type WifiForm = ReturnType<typeof useWifiForm>
