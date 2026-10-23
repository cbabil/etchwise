/**
 * Upstream SshKeyManager "Browse": a native open dialog filtered to
 * public key files, then `readFileContents(path)`. Outside Tauri (browser
 * preview, tests) there is no native dialog and nothing is picked.
 */
import { call } from '@/api/imager'

import type { Tr } from './sshKeys'

const inTauri = (): boolean => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

/** Returns the picked file's contents, or '' when cancelled or unreadable. */
export async function browseKeyFile(tr: Tr, common: Tr): Promise<string> {
  if (!inTauri()) return ''
  const { open } = await import('@tauri-apps/plugin-dialog')
  const picked = await open({
    title: tr('Select SSH Public Key'),
    multiple: false,
    directory: false,
    filters: [
      { name: common('Public Key files (*.pub)'), extensions: ['pub'] },
      { name: common('All files (*)'), extensions: ['*'] },
    ],
  })
  if (typeof picked !== 'string' || picked.length === 0) return ''
  try {
    return String((await call<string | null>('readFileContents', picked)) ?? '')
  } catch {
    return ''
  }
}
