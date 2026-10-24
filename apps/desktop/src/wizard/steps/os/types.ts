/**
 * Shapes of the backend's device and OS rows (upstream `HWListModel` /
 * `OSListModel` roles, snake_case) and the row predicates of upstream
 * `OSSelectionStep.qml`.
 */

/** One `getHWList().devices` row. */
export interface HwDevice {
  name: string
  description: string
  icon: string
  tags?: readonly string[]
  capabilities?: readonly string[]
  matching_type?: string
  architecture?: string
  isUsbBootConnected?: boolean
  /** "other" for devices merged from an extra repository. */
  group?: string
}

export interface HwList {
  devices: readonly HwDevice[]
  currentIndex: number
}

/** One `getOSList()` / `getSublist()` row; every field may be missing. */
export interface OsEntry {
  name: string
  description: string
  icon?: string
  url?: string
  release_date?: string
  extract_sha256?: string
  extract_size?: number
  image_download_size?: number
  contains_multiple_files?: boolean
  init_format?: string
  bmap_url?: string
  capabilities?: readonly string[]
  devices?: readonly string[]
  subitems_json?: string
  subitems_url?: string
  recommended?: boolean
}

export const URL_CUSTOM = 'internal://custom'
export const URL_FORMAT = 'internal://format'
export const URL_BACK = 'internal://back'

/** Upstream `isOSsublist`: a group with inline sub-items or a sub-list URL. */
export function isSublist(entry: OsEntry): boolean {
  const json = typeof entry.subitems_json === 'string' && entry.subitems_json !== ''
  const url = entry.subitems_url
  return json || (typeof url === 'string' && url !== '' && url !== URL_BACK)
}

export const isBackRow = (entry: OsEntry): boolean => entry.subitems_url === URL_BACK

/** The "Back" row heading every sublist; its text is translated when rendered. */
export const BACK_ROW: OsEntry = {
  name: 'Back',
  description: 'Go back to main menu',
  icon: '../icons/ic_chevron_left_40px.svg',
  url: '',
  release_date: '',
  subitems_url: URL_BACK,
  subitems_json: '',
}
