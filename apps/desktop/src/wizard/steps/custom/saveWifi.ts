/**
 * Upstream Wi-Fi `onNextClicked`: stores `wifiMode`, the SSID, the PSK
 * (a passphrase is PBKDF2-hashed against the SSID, a 64-hex key kept
 * as-is; the saved crypt is kept when the password is blank and the SSID
 * unchanged) and `wifiHidden`, at runtime and in the saved settings.
 */
import { call } from '@/api/imager'

import type { CustomizationSettings } from '../../state'
import type { WizardApi } from '../../WizardContext'
import { editSettings, rewriteSaved, str } from './settings'
import type { WifiFields } from './useWifiForm'
import { isPassphrase, ssidUnchanged } from './validation'

interface WifiChange {
  patch: CustomizationSettings
  remove: string[]
  configured: boolean
}

/** The settings change for `fields`; `crypt` is the new PSK hash, or null when none was typed. */
export function wifiChange(fields: WifiFields, crypt: string | null, keep: boolean): WifiChange {
  const ssid = fields.ssid.trim()
  if (ssid.length === 0) {
    return {
      patch: { wifiMode: fields.mode },
      remove: ['wifiSSID', 'wifiPasswordCrypt', 'wifiHidden'],
      configured: false,
    }
  }
  const patch: Record<string, unknown> = {
    wifiMode: fields.mode,
    wifiSSID: ssid,
    wifiHidden: fields.hidden,
  }
  const remove: string[] = []
  if (fields.mode === 'secure' && crypt !== null) patch.wifiPasswordCrypt = crypt
  else if (fields.mode === 'open' || !keep) remove.push('wifiPasswordCrypt')
  return { patch, remove, configured: true }
}

/** Returns false (nothing saved) in upstream's "passwords differ" safety case. */
export async function saveWifi(wizard: WizardApi, fields: WifiFields): Promise<boolean> {
  const ssid = fields.ssid.trim()
  const pwd = fields.password
  const secureWithPassword = ssid.length > 0 && fields.mode === 'secure' && pwd.length > 0
  if (secureWithPassword && pwd !== fields.confirm) return false
  const crypt = secureWithPassword
    ? isPassphrase(pwd)
      ? str(await call('pbkdf2', pwd, ssid))
      : pwd
    : null
  const runtime = wizard.customizationSettings
  const keep = Boolean(runtime.wifiPasswordCrypt) && ssidUnchanged(ssid, str(runtime.wifiSSID))
  const change = wifiChange(fields, crypt, keep)
  editSettings(wizard, change.patch, change.remove)
  wizard.update({ wifiConfigured: change.configured })
  await rewriteSaved(change.patch, change.remove)
  return true
}
