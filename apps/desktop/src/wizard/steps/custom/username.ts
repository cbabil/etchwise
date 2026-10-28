/**
 * The username rule (not in upstream): `USERNAME_RE` (at most 32
 * characters) and not a name the device keeps for a system account or
 * group. Kura's first boot refuses those and then ignores its whole setup
 * file, so the device keeps its default login. `is_safe_username` in
 * `imager-core` applies the same rule (a Rust test keeps the lists equal).
 */

import { USERNAME_RE } from './validation'

/** Debian system account and group names, plus those of common packages. */
// reserved-usernames:start
export const RESERVED_USERNAMES: readonly string[] = [
  '_apt',
  '_chrony',
  '_rpc',
  '_ssh',
  'adm',
  'audio',
  'avahi',
  'avahi-autoipd',
  'backup',
  'bin',
  'bluetooth',
  'cdrom',
  'colord',
  'crontab',
  'daemon',
  'dialout',
  'dip',
  'disk',
  'dnsmasq',
  'docker',
  'fax',
  'floppy',
  'games',
  'gnats',
  'gpio',
  'i2c',
  'input',
  'irc',
  'kmem',
  'kvm',
  'list',
  'lp',
  'lpadmin',
  'mail',
  'man',
  'messagebus',
  'netdev',
  'news',
  'nobody',
  'nogroup',
  'ntp',
  'operator',
  'plugdev',
  'polkitd',
  'proxy',
  'pulse',
  'rdma',
  'render',
  'root',
  'sasl',
  'sgx',
  'shadow',
  'spi',
  'src',
  'ssl-cert',
  'sshd',
  'staff',
  'statd',
  'sudo',
  'sync',
  'sys',
  'syslog',
  'tape',
  'tss',
  'tty',
  'users',
  'utmp',
  'uucp',
  'uuidd',
  'video',
  'voice',
  'www-data',
]
// reserved-usernames:end

/** Names under this prefix belong to systemd (systemd-network, systemd-journal...). */
const RESERVED_PREFIX = 'systemd-'

export const RESERVED_MESSAGE = 'This name is used by the system; choose another.'

export function isReservedUsername(name: string): boolean {
  return RESERVED_USERNAMES.includes(name) || name.startsWith(RESERVED_PREFIX)
}

/** A name the device accepts for the first user. */
export function isValidUsername(name: string): boolean {
  return USERNAME_RE.test(name) && !isReservedUsername(name)
}

/** The untranslated inline error for the field, or null for none. */
export function usernameError(name: string): string | null {
  return isReservedUsername(name) ? RESERVED_MESSAGE : null
}
