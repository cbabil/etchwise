/**
 * Native open-file dialog (upstream `getNativeOpenFileName`) through the
 * Tauri dialog plugin. Outside Tauri (browser preview, tests) it answers
 * null, as if the user cancelled.
 */
export interface FileFilter {
  name: string
  extensions: string[]
}

export interface PickFileOptions {
  title: string
  filters: FileFilter[]
  /** Starting folder below the home directory, e.g. `.ssh`. */
  homeSubdir?: string
}

const inTauri = (): boolean => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

async function startFolder(subdir: string | undefined): Promise<string | undefined> {
  if (!subdir) return undefined
  try {
    const { homeDir, join } = await import('@tauri-apps/api/path')
    return await join(await homeDir(), subdir)
  } catch {
    return undefined
  }
}

/** Resolves to the chosen path, or null when cancelled or unavailable. */
export async function pickFile(options: PickFileOptions): Promise<string | null> {
  if (!inTauri()) return null
  const { open } = await import('@tauri-apps/plugin-dialog')
  const defaultPath = await startFolder(options.homeSubdir)
  const picked = await open({
    title: options.title,
    filters: options.filters,
    multiple: false,
    directory: false,
    ...(defaultPath ? { defaultPath } : {}),
  })
  return typeof picked === 'string' && picked.length > 0 ? picked : null
}
