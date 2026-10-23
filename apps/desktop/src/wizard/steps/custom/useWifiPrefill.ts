/**
 * Start-up prefill of upstream `WifiCustomizationStep.qml`: when no SSID
 * is saved, detect the current one (`getSSID`), retrying on
 * `locationPermissionGranted`; when no crypt is saved, offer to read the
 * PSK from the system (`getPSKForSSID`). On macOS that reads the keychain,
 * so upstream first asks through `KeychainPermissionDialog` (granted
 * automatically when warnings are disabled).
 */
import { useEffect, useRef, useState } from 'react'

import { call, onImagerEvent } from '@/api/imager'

import { useWizard } from '../../WizardContext'
import { str } from './settings'
import { useMountEffect } from './useMountEffect'
import type { WifiForm } from './useWifiForm'

/** The macOS keychain prompts for admin credentials, so consent is asked first there only. */
export const keychainNeedsConsent = (): boolean =>
  typeof navigator !== 'undefined' && /Mac/i.test(navigator.userAgent)

export function useWifiPrefill(form: WifiForm) {
  const wizard = useWizard()
  const [askSsid, setAskSsid] = useState<string | null>(null)
  const latest = useRef(form)
  useEffect(() => {
    latest.current = form
  })

  const fillPsk = async (ssid: string) => {
    const psk = str(await call('getPSKForSSID', ssid))
    const now = latest.current.fields
    if (psk && now.mode === 'secure' && now.password.length === 0)
      latest.current.set({ password: psk, confirm: psk })
  }
  const requestPsk = (ssid: string) => {
    if (form.hadSavedCrypt || !ssid) return
    if (keychainNeedsConsent() && !wizard.disableWarnings) setAskSsid(ssid)
    else void fillPsk(ssid)
  }
  const detect = async () => {
    const detected = str(await call('getSSID'))
    if (!detected || latest.current.fields.ssid.length > 0) return
    latest.current.set({ ssid: detected })
    requestPsk(detected)
  }

  useMountEffect(() => {
    const saved = form.fields.ssid
    if (saved) requestPsk(saved)
    else void detect()
    let off: (() => void) | null = null
    let live = true
    void onImagerEvent((event) => {
      if (event.name === 'locationPermissionGranted' && !latest.current.fields.ssid) void detect()
    }).then((unsubscribe) => (live ? (off = unsubscribe) : unsubscribe()))
    return () => {
      live = false
      off?.()
    }
  })

  const respond = (granted: boolean) => {
    const ssid = askSsid
    setAskSsid(null)
    if (granted && ssid) void fillPsk(ssid)
  }
  return { keychainOpen: askSsid !== null, respond }
}
