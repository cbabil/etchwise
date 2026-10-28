/**
 * Pure validation rules of the customisation steps, verbatim from the
 * upstream QML (hostname / username validators, the Wi-Fi PSK rules).
 */

export const HOSTNAME_RE = /^[a-zA-Z0-9][a-zA-Z0-9-]{0,62}$/
/** Upstream's pattern, capped at 32 characters (Kura's and useradd's limit). */
export const USERNAME_RE = /^[a-z_][a-z0-9_-]{0,31}$/

/**
 * Qt `RegularExpressionValidator` input restriction: an edit is accepted
 * only when the result is empty or matches (every prefix of a valid
 * hostname / username is itself valid, so "intermediate" equals "valid").
 */
export function acceptsInput(re: RegExp, text: string): boolean {
  return text.length === 0 || re.test(text)
}

export function isAsciiPrintable(text: string): boolean {
  for (let i = 0; i < text.length; i++) {
    const code = text.charCodeAt(i)
    if (code < 32 || code > 126) return false
  }
  return true
}

export function isHex64(text: string): boolean {
  return /^[0-9a-fA-F]{64}$/.test(text)
}

/** Upstream `isValidWifiPassword`: empty (open), 64 hex, or 8-63 printable ASCII. */
export function isValidWifiPassword(text: string): boolean {
  if (text.length === 0) return true
  if (isHex64(text)) return true
  return text.length >= 8 && text.length <= 63 && isAsciiPrintable(text)
}

/** Upstream `ssidUnchanged`. */
export function ssidUnchanged(ssid: string, prev: string): boolean {
  return (ssid || '') === (prev || '')
}

export type WifiMode = 'secure' | 'open'

export interface WifiPasswordState {
  mode: WifiMode
  password: string
  confirm: string
  /** A saved crypt exists and the SSID is unchanged. */
  canKeep: boolean
}

/** Upstream `passwordErrorMessage()`: the untranslated message, or null for none. */
export function wifiPasswordError(s: WifiPasswordState): string | null {
  if (s.mode === 'open') return null
  if (s.canKeep && s.password.length === 0) return null
  const pwd = s.password
  if (pwd.length === 0) return 'Enter a password'
  if (!isHex64(pwd)) {
    if (pwd.length < 8) return 'Password is too short (min 8 characters)'
    if (pwd.length > 63) return 'Password is too long (max 63 characters)'
    if (!isAsciiPrintable(pwd)) return 'Password contains unsupported characters'
  }
  if (pwd !== s.confirm) return "Passwords don't match"
  return null
}

/** Upstream Wi-Fi `nextButtonEnabled`. */
export function wifiNextEnabled(ssid: string, s: WifiPasswordState): boolean {
  if (ssid.trim().length === 0) return true
  if (s.mode === 'open') return true
  if (s.canKeep && s.password.length === 0) return true
  if (s.password.length === 0) return false
  if (!isValidWifiPassword(s.password)) return false
  return s.password === s.confirm
}

/** Upstream `pwd.length >= 8 && pwd.length < 64`: passphrases are PBKDF2-hashed, 64-hex PSKs kept. */
export function isPassphrase(pwd: string): boolean {
  return pwd.length >= 8 && pwd.length < 64
}
