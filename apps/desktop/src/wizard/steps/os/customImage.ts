/**
 * "Use custom" of upstream `OSSelectionStep.qml`: the native file dialog
 * (upstream's image filters) and the `onFileSelected` handler that makes
 * the picked file the source and clears customisation it cannot take.
 */
import { open } from '@tauri-apps/plugin-dialog'

import { call } from '@/api/imager'

import type { SelectionWizard } from './selectImage'

const IMAGE_EXTENSIONS = ['img', 'zip', 'iso', 'gz', 'xz', 'zst', 'wic'] as const

const inTauri = (): boolean => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export interface DialogLabels {
  title: string
  /** Upstream `CommonStrings.imageFiltersList[0]`, "Image files (*.img …)". */
  images: string
  /** "All files (*)". */
  all: string
}

export const imageFilterPattern = IMAGE_EXTENSIONS.map((e) => `*.${e}`).join(' ')

/** Opens the native picker; null when cancelled or outside Tauri. */
export async function chooseImageFile(labels: DialogLabels): Promise<string | null> {
  if (!inTauri()) return null
  const picked = await open({
    title: labels.title,
    multiple: false,
    directory: false,
    filters: [
      { name: labels.images, extensions: [...IMAGE_EXTENSIONS] },
      { name: labels.all, extensions: ['*'] },
    ],
  })
  return typeof picked === 'string' && picked ? picked : null
}

/** `file://` URL of a local path (Windows drive paths get three slashes). */
export function toFileUrl(path: string): string {
  if (path.startsWith('file://')) return path
  const slashed = path.replace(/\\/g, '/')
  return slashed.startsWith('/') ? `file://${slashed}` : `file:///${slashed}`
}

/** Upstream `onFileSelected(fileUrl)`; returns the source size for the status line. */
export async function applyCustomImage(wizard: SelectionWizard, path: string): Promise<number> {
  await call('setSrc', { url: toFileUrl(path) })
  const [name, customization, forced, size] = await Promise.all([
    call<string | null>('srcFileName'),
    call<boolean | null>('imageSupportsCustomization'),
    call<boolean | null>('isSecureBootForcedByCliFlag'),
    call<number | null>('getSelectedSourceSize'),
  ])
  wizard.setSelectedOsName(name ?? '')
  wizard.update({
    autoinstall: false,
    ...(customization
      ? { customizationSupported: true }
      : {
          customizationSupported: false,
          hostnameConfigured: false,
          localeConfigured: false,
          userConfigured: false,
          wifiConfigured: false,
          sshEnabled: false,
          piConnectEnabled: false,
          piConnectAvailable: false,
          secureBootAvailable: Boolean(forced),
        }),
  })
  return size ?? 0
}
