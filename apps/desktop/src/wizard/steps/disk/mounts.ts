/**
 * Which folders Kura can put on their own volume, mirroring
 * `validate_storage` in cbabil/kura's `firstboot-storage-check.sh`: any plain
 * absolute folder except the system's own (`STORAGE_SYSTEM`), Kura's state
 * and what is under /tmp; and each volume's name (`lv_name`).
 */

/** Folders offered in the folder picker, in the order rows are added. */
export const SUGGESTED_MOUNTS = [
  '/var',
  '/var/lib',
  '/var/log',
  '/var/cache',
  '/var/tmp',
  '/var/lib/docker',
  '/tmp',
  '/home',
  '/srv',
  '/opt',
]

/** Folders that stay on the root, with everything under them. */
const SYSTEM = [
  '/boot',
  '/etc',
  '/usr',
  '/bin',
  '/sbin',
  '/lib',
  '/lib32',
  '/lib64',
  '/dev',
  '/proc',
  '/sys',
  '/run',
  '/root',
  '/var/run',
  '/var/lock',
  '/var/lib/kura',
]

const MAX_LENGTH = 64
const PLAIN_RE = /^(\/[A-Za-z0-9._-]+)+$/

const isPlain = (mount: string): boolean =>
  PLAIN_RE.test(mount) && mount.split('/').every((part) => part !== '.' && part !== '..')

const under = (mount: string, dir: string): boolean => mount === dir || mount.startsWith(`${dir}/`)

/** Volume names LVM refuses or keeps for its own volumes (Kura's `lvm_refuses`). */
const LVM_RESERVED_RE =
  /_(cdata|cmeta|corig|mimage|mlog|pmspare|rimage|rmeta|tdata|tmeta|vorigin|vdata)/

const lvmRefuses = (mount: string): boolean => {
  const name = volumeName(mount)
  return (
    mount.split('/').some((part) => part.startsWith('-')) ||
    name === 'snapshot' ||
    name === 'pvmove' ||
    LVM_RESERVED_RE.test(name)
  )
}

/** The folder's problem as Kura sees it, or '' when it can have a volume. */
export function mountError(mount: string): string {
  if (!isPlain(mount)) return 'Use a folder such as /srv/media.'
  if (mount.length > MAX_LENGTH) return 'Folders go up to 64 characters.'
  const system = SYSTEM.find((dir) => under(mount, dir))
  if (system) return `${system} stays on the system disk.`
  if (mount.startsWith('/tmp/')) return 'Folders under /tmp stay on the system disk.'
  if (lvmRefuses(mount)) return 'LVM keeps this name for itself; choose another folder.'
  return ''
}

/** The volume Kura makes for a folder: `/var/lib` is `var-lib`. */
export function volumeName(mount: string): string {
  return mount.replace(/^\//, '').replaceAll('/', '-')
}

/** The first clash between volume names, or ''. */
export function nameClash(mounts: readonly string[], swap: boolean): string {
  const seen = new Map<string, string>()
  for (const mount of mounts) {
    const other = seen.get(volumeName(mount))
    if (other !== undefined) return `${other} and ${mount} would share a volume name.`
    seen.set(volumeName(mount), mount)
  }
  return swap && seen.has('swap') ? "/swap would share the swap volume's name." : ''
}
