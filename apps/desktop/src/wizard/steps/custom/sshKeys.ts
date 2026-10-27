/**
 * Pure helpers of upstream `SshKeyManager.qml` and the key loading in
 * `RemoteAccessStep.qml`: split / dedupe keys, the shortened display form
 * and the per-key accessible names. Browsing uses the Tauri dialog plugin.
 */

const BEGIN_RFC4716 = /^---- BEGIN SSH2 PUBLIC KEY ----$/
const END_RFC4716 = /^---- END SSH2 PUBLIC KEY ----$/
const OPENSSH_KEY = /^(?:ssh-|ecdsa-|sk-)[\w@.-]+ \S/

/**
 * Upstream `splitKeys`: one key per non-empty trimmed line, plus (upstream
 * 2.0.8) the multi-line RFC 4716 blocks PuTTY saves, each turned into one
 * OpenSSH line.
 */
export function splitKeys(text: string): string[] {
  const keys: string[] = []
  let block: string[] | null = null
  for (const line of text.split(/\r?\n/).map((l) => l.trim())) {
    if (block === null) {
      if (BEGIN_RFC4716.test(line)) block = []
      else if (line) keys.push(line)
    } else if (OPENSSH_KEY.test(line)) {
      // A block that never ended: drop it, keep this key and the rest
      keys.push(line)
      block = null
    } else if (END_RFC4716.test(line)) {
      const key = rfc4716Key(block)
      if (key) keys.push(key)
      block = null
    } else {
      block.push(line)
    }
  }
  return keys
}

/**
 * One RFC 4716 block's lines between BEGIN and END: "Tag: value" headers
 * (continued while a line ends in a backslash), then the key data. The type
 * comes from the data itself, so ed25519 keys are not mistaken for RSA
 * (upstream always writes "ssh-rsa").
 */
function rfc4716Key(lines: readonly string[]): string | null {
  let comment = ''
  let i = 0
  while (i < lines.length && /^[\w-]+:/.test(lines[i] ?? '')) {
    let header = lines[i] ?? ''
    while (header.endsWith('\\') && i + 1 < lines.length) header = header.slice(0, -1) + lines[++i]
    const colon = header.indexOf(':')
    if (header.slice(0, colon).toLowerCase() === 'comment') {
      comment = header
        .slice(colon + 1)
        .trim()
        .replace(/^"(.*)"$/, '$1')
    }
    i++
  }
  const data = lines.slice(i).join('')
  const type = blobKeyType(data)
  return type ? [type, data, comment].filter(Boolean).join(' ') : null
}

/** The key type a public key blob names first (RFC 4253: length, then name). */
function blobKeyType(base64: string): string | null {
  let bytes: string
  try {
    bytes = atob(base64)
  } catch {
    return null
  }
  if (bytes.length < 4) return null
  const length =
    ((bytes.charCodeAt(0) << 24) |
      (bytes.charCodeAt(1) << 16) |
      (bytes.charCodeAt(2) << 8) |
      bytes.charCodeAt(3)) >>>
    0
  if (length < 1 || length > 64 || 4 + length > bytes.length) return null
  const type = bytes.slice(4, 4 + length)
  return /^[a-z0-9@.-]+$/.test(type) ? type : null
}

/** Upstream `deduplicateKeys`: first occurrence wins. */
export function dedupeKeys(keys: readonly string[]): string[] {
  return [...new Set(keys)]
}

/** Upstream `addKey`: appends the trimmed key unless empty or already present. */
export function addKey(keys: readonly string[], key: string): readonly string[] {
  const trimmed = key.trim()
  if (trimmed.length === 0 || keys.includes(trimmed)) return keys
  return [...keys, trimmed]
}

interface KeyParts {
  type: string
  data: string
  comment: string
}

function parts(key: string): KeyParts | null {
  const fields = key.split(/\s+/)
  if (fields.length < 2) return null
  return { type: fields[0] ?? '', data: fields[1] ?? '', comment: fields.slice(2).join(' ') }
}

/** Upstream display text: "type start...end comment" (malformed keys shortened 15...15). */
export function displayKey(key: string): string {
  const p = parts(key)
  if (!p)
    return key.length > 40 ? `${key.substring(0, 15)}...${key.substring(key.length - 15)}` : key
  const data =
    p.data.length > 20
      ? `${p.data.substring(0, 8)}...${p.data.substring(p.data.length - 8)}`
      : p.data
  return [p.type, data, p.comment].filter(Boolean).join(' ')
}

export type Tr = (source: string, ...args: unknown[]) => string

/** Upstream row `Accessible.name`. */
export function keyAccessibleName(tr: Tr, key: string, index: number): string {
  const p = parts(key)
  if (!p) return tr('SSH key %1', index + 1)
  return p.comment
    ? tr('SSH key %1: %2, %3', index + 1, p.type, p.comment)
    : tr('SSH key %1: %2', index + 1, p.type)
}

/** Upstream Remove button description. */
export function removeDescription(tr: Tr, key: string, index: number): string {
  const p = parts(key)
  return p?.comment ? tr('Remove SSH key: %1', p.comment) : tr('Remove SSH key %1', index + 1)
}

/** Keys from `sshAuthorizedKeys`, falling back to the legacy `sshPublicKey`. */
export function savedKeys(settings: Readonly<Record<string, unknown>>): string[] {
  const authorized = settings.sshAuthorizedKeys
  if (typeof authorized === 'string' && authorized) return splitKeys(authorized)
  const legacy = settings.sshPublicKey
  return typeof legacy === 'string' ? splitKeys(legacy) : []
}
