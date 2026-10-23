/**
 * Pure helpers of upstream `SshKeyManager.qml` and the key loading in
 * `RemoteAccessStep.qml`: split / dedupe keys, the shortened display form
 * and the per-key accessible names. Browsing uses the Tauri dialog plugin.
 */

/** Upstream `splitKeys`: non-empty trimmed lines. */
export function splitKeys(text: string): string[] {
  if (!text) return []
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
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
