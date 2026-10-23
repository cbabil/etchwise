/**
 * Shapes of the backend `getDriveList` answer (upstream `DriveListModel`
 * roles) and the pure rules of `StorageSelectionStep.qml` built on them:
 * which rows are hidden or selectable and the screen-reader status text.
 */
import { z } from 'zod'

const driveSchema = z.object({
  key: z.string(),
  device: z.string(),
  description: z.string().catch(''),
  size: z.number().catch(0),
  isUsb: z.boolean().catch(false),
  isScsi: z.boolean().catch(false),
  isReadOnly: z.boolean().catch(false),
  isSystem: z.boolean().catch(false),
  isRpiboot: z.boolean().catch(false),
  mountpoints: z.array(z.string()).catch([]),
  childDevices: z.array(z.string()).catch([]),
})

const driveListSchema = z.object({
  drives: z.array(driveSchema).catch([]),
  error: z.string().catch(''),
})

export type DriveInfo = z.infer<typeof driveSchema>
export type DriveList = z.infer<typeof driveListSchema>

export const emptyDriveList: DriveList = { drives: [], error: '' }

/** Validates a backend answer; anything malformed becomes an empty list. */
export function parseDriveList(value: unknown): DriveList {
  const parsed = driveListSchema.safeParse(value)
  return parsed.success ? parsed.data : emptyDriveList
}

/** Rows shown: system drives are hidden while "Exclude system drives" is on. */
export const visibleDrives = (drives: readonly DriveInfo[], filter: boolean): DriveInfo[] =>
  drives.filter((d) => !(d.isSystem && filter))

/** Upstream `unselectable`: read-only rows, except rpiboot devices. */
export const unselectable = (d: Pick<DriveInfo, 'isReadOnly' | 'isRpiboot'>): boolean =>
  d.isReadOnly && !d.isRpiboot

/** The name shown for a selected drive (upstream `description || device`). */
export const driveName = (d: Pick<DriveInfo, 'description' | 'device'>): string =>
  d.description || d.device

export type StorageStatus = 'ok' | 'error' | 'none' | 'readOnlyFiltered' | 'readOnly' | 'hidden'

/** Upstream `updateStorageStatus` folded into one value. */
export function storageStatus(list: DriveList, filter: boolean): StorageStatus {
  if (list.error) return 'error'
  const { drives } = list
  if (visibleDrives(drives, filter).some((d) => !unselectable(d))) return 'ok'
  if (drives.length === 0) return 'none'
  if (drives.every((d) => d.isReadOnly)) return filter ? 'readOnlyFiltered' : 'readOnly'
  return 'hidden'
}

type Tr = (source: string, ...args: unknown[]) => string

/** Upstream `getStorageStatusMessage()` (read by screen readers only). */
export function storageStatusMessage(status: StorageStatus, error: string, tr: Tr): string {
  switch (status) {
    case 'error':
      return tr(
        'Could not list storage devices: %1. This may be a permissions issue. Try running the application with administrator privileges.',
        error,
      )
    case 'none':
      return tr('No storage devices found. Please connect a storage device to continue.')
    case 'readOnlyFiltered':
      return tr(
        "No valid storage devices are currently available. All visible devices are read-only. Try connecting a new storage device, or uncheck 'Exclude system drives' to show hidden system drives.",
      )
    case 'readOnly':
      return tr(
        'No valid storage devices are currently available. All devices are read-only. Please connect a writable storage device to continue.',
      )
    case 'hidden':
      return tr(
        "No valid storage devices are currently available. Uncheck 'Exclude system drives' to show hidden system drives, or connect a new storage device.",
      )
    default:
      return ''
  }
}

/** Upstream list `accessibleName`: name, selectable count, hint and status. */
export function listAccessibleName(selectable: number, statusMessage: string, tr: Tr): string {
  const count =
    selectable === 0
      ? tr('No devices')
      : selectable === 1
        ? tr('1 device')
        : tr('%1 devices', selectable)
  const parts = [
    tr('Storage device list'),
    count,
    tr('Use arrow keys to navigate, Enter or Space to select'),
  ]
  if (statusMessage) parts.push(statusMessage)
  return parts.join('. ')
}
