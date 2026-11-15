/**
 * Kura's disk layout (Etchwise, not upstream): the presets, the rules Kura
 * applies on first boot (`firstboot-storage-check.sh` in cbabil/kura), and
 * the `storage` object sent in `kura-setup.json` (setting `kuraStorage`).
 * Sizes are edited as text and are whole GiB.
 */
import type { CustomizationSettings } from '../../state'
import { mountError, nameClash, SUGGESTED_MOUNTS } from './mounts'

export { mountError, SUGGESTED_MOUNTS } from './mounts'

/** Translation context of the step and its parts. */
export const DISK_CONTEXT = 'DiskLayoutStep'

export type Preset = 'simple' | 'server' | 'custom'

export interface VolumeRow {
  mount: string
  size: string
  rest: boolean
}

export interface Layout {
  preset: Preset
  root: string
  tmp: string
  /** Swap size in GiB; '' for none. */
  swap: string
  volumes: readonly VolumeRow[]
}

export interface Storage {
  rootGiB: number
  tmpGiB?: number
  swapGiB?: number
  volumes: { mount: string; sizeGiB?: number }[]
}

const GIB = 1024 ** 3
/** The boot partition and LVM's room, before the root. */
const OVERHEAD = 274 * 1024 ** 2
const MAX_GIB = 4096

const row = (mount: string, size: string): VolumeRow => ({ mount, size, rest: size === '' })

/** Logs, temporary files and homes on their own volumes; Docker on the rest. */
export const SERVER: Layout = {
  preset: 'server',
  root: '16',
  tmp: '1',
  swap: '',
  volumes: [
    row('/var/log', '4'),
    row('/var/tmp', '2'),
    row('/home', '8'),
    row('/var/lib/docker', ''),
  ],
}

/** `layout` under another preset: Server resets the sizes, the others keep them. */
export function withPreset(layout: Layout, preset: Preset): Layout {
  return preset === 'server' ? SERVER : { ...layout, preset }
}

const whole = (text: string): number => (/^\d+$/.test(text) ? Number(text) : 0)

/** Whether /tmp has its own volume rather than a RAM disk. */
export const tmpOnDisk = (l: Layout): boolean => l.volumes.some((v) => v.mount === '/tmp')

/** The first problem with the sizes (root, /tmp, swap), or ''. */
function sizeError(l: Layout): string {
  const sizes = [l.root, l.tmp, l.swap, ...l.volumes.filter((v) => !v.rest).map((v) => v.size)]
  if (sizes.some((s) => whole(s) > MAX_GIB)) return 'Sizes go up to 4096 GiB.'
  if (whole(l.root) < 4) return 'The root needs at least 4 GiB.'
  if (!tmpOnDisk(l) && whole(l.tmp) < 1) return '/tmp needs at least 1 GiB.'
  if (l.swap !== '' && whole(l.swap) < 1) return 'Swap needs at least 1 GiB, or leave it empty.'
  return ''
}

/** The first problem, or '' when Kura would accept the layout. */
export function layoutError(l: Layout): string {
  if (l.preset === 'simple') return ''
  const sizes = sizeError(l)
  if (sizes) return sizes
  if (l.volumes.length === 0 && l.swap === '') return 'Add at least one folder, or choose Simple.'
  const mounts = l.volumes.map((v) => v.mount)
  const folder = mounts.map(mountError).find((e) => e !== '')
  if (folder) return folder
  if (l.volumes.some((v) => !v.rest && whole(v.size) < 1))
    return 'Give each folder a size, or let one take the rest of the disk.'
  if (new Set(mounts).size < mounts.length) return 'Each folder can have only one volume.'
  const clash = nameClash(mounts, l.swap !== '')
  if (clash) return clash
  if (l.volumes.filter((v) => v.rest).length > 1)
    return 'Only one folder can take the rest of the disk.'
  return ''
}

/** What Kura receives, or null to keep one root that fills the disk. */
export function toStorage(l: Layout): Storage | null {
  if (l.preset === 'simple') return null
  const swap = whole(l.swap)
  return {
    rootGiB: whole(l.root),
    ...(tmpOnDisk(l) ? {} : { tmpGiB: whole(l.tmp) }),
    ...(swap > 0 ? { swapGiB: swap } : {}),
    volumes: l.volumes.map((v) =>
      v.rest ? { mount: v.mount } : { mount: v.mount, sizeGiB: whole(v.size) },
    ),
  }
}

/** The smallest drive the layout fits on (0 for Simple). */
export function neededBytes(l: Layout): number {
  const storage = toStorage(l)
  if (!storage) return 0
  const fixed = storage.volumes.reduce((sum, v) => sum + (v.sizeGiB ?? 1), 0)
  return (storage.rootGiB + (storage.swapGiB ?? 0) + fixed) * GIB + OVERHEAD
}

/** A row for the first suggested folder not listed yet, otherwise an empty one to type in. */
export function addRow(l: Layout): Layout {
  const free = SUGGESTED_MOUNTS.find((m) => !l.volumes.some((v) => v.mount === m)) ?? ''
  return { ...l, volumes: [...l.volumes, { mount: free, size: '1', rest: false }] }
}

const asText = (value: unknown, fallback: string): string =>
  typeof value === 'number' && Number.isInteger(value) ? String(value) : fallback

function savedVolumes(value: unknown): VolumeRow[] {
  if (!Array.isArray(value)) return [...SERVER.volumes]
  return value.flatMap((v: unknown) => {
    const item = v as { mount?: unknown; sizeGiB?: unknown } | null
    return typeof item?.mount === 'string' ? [row(item.mount, asText(item.sizeGiB, ''))] : []
  })
}

/** The layout saved with the settings (`kuraDiskLayout`, `kuraStorage`). */
export function fromSettings(settings: CustomizationSettings): Layout {
  const preset = settings.kuraDiskLayout
  if (preset === 'server') return SERVER
  if (preset !== 'custom') return { ...SERVER, preset: 'simple' }
  const saved = (settings.kuraStorage ?? {}) as {
    rootGiB?: unknown
    tmpGiB?: unknown
    swapGiB?: unknown
    volumes?: unknown
  }
  return {
    preset: 'custom',
    root: asText(saved.rootGiB, SERVER.root),
    tmp: asText(saved.tmpGiB, SERVER.tmp),
    swap: asText(saved.swapGiB, ''),
    volumes: savedVolumes(saved.volumes),
  }
}
