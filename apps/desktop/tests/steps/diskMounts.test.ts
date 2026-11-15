import { mountError, SUGGESTED_MOUNTS } from '@/wizard/steps/disk/layout'

// Mirrors the validate_storage cases in cbabil/kura's tests/shell/firstboot-storage.bats
describe('Kura disk layout folders', () => {
  it('suggests only folders Kura accepts', () => {
    for (const m of SUGGESTED_MOUNTS) expect(mountError(m)).toBe('')
  })

  it('accepts any plain folder, as Kura does', () => {
    for (const m of [
      '/var',
      '/var/lib',
      '/var/cache',
      '/tmp',
      '/echange',
      '/data/media',
      '/srv/my_app.d',
    ])
      expect(mountError(m)).toBe('')
  })

  it('rejects a folder that is not a plain absolute path', () => {
    const bad = ['', 'home', '/', '/home/', '//home', '/etc/../home', '/a/./b', '/a b', '/a;b']
    for (const m of bad) expect(mountError(m)).toBe('Use a folder such as /srv/media.')
    expect(mountError(`/${'x'.repeat(64)}`)).toBe('Folders go up to 64 characters.')
    expect(mountError(`/${'x'.repeat(63)}`)).toBe('')
  })

  it('rejects names LVM refuses or keeps for itself', () => {
    for (const m of [
      '/-data',
      '/srv/-x',
      '/snapshot',
      '/pvmove',
      '/data_tmeta',
      '/a/b_rimage_1',
      '/x_cdata',
    ])
      expect(mountError(m)).toBe('LVM keeps this name for itself; choose another folder.')
    expect(mountError('/srv/snapshots')).toBe('')
    expect(mountError('/data-x')).toBe('')
  })

  it('keeps system folders, the state and folders under /tmp on the root', () => {
    const system = ['/boot', '/etc', '/usr', '/bin', '/sbin', '/lib', '/lib32', '/lib64', '/dev']
    for (const m of [...system, '/proc', '/sys', '/run', '/root', '/var/run', '/var/lock'])
      expect(mountError(m)).toBe(`${m} stays on the system disk.`)
    expect(mountError('/boot/efi')).toBe('/boot stays on the system disk.')
    expect(mountError('/etc/ssh')).toBe('/etc stays on the system disk.')
    expect(mountError('/usr/local')).toBe('/usr stays on the system disk.')
    expect(mountError('/var/lib/kura')).toBe('/var/lib/kura stays on the system disk.')
    expect(mountError('/var/lib/kura/x')).toBe('/var/lib/kura stays on the system disk.')
    expect(mountError('/tmp/x')).toBe('Folders under /tmp stay on the system disk.')
    expect(mountError('/etcetera')).toBe('')
    expect(mountError('/var/libx')).toBe('')
  })
})
