import {
  addRow,
  fromSettings,
  layoutError,
  neededBytes,
  SERVER,
  toStorage,
  withPreset,
} from '@/wizard/steps/disk/layout'

const GIB = 1024 ** 3

describe('Kura disk layout', () => {
  it('server preset: the root, /tmp, logs, temp files, homes and Docker on the rest', () => {
    expect(toStorage(SERVER)).toEqual({
      rootGiB: 16,
      tmpGiB: 1,
      volumes: [
        { mount: '/var/log', sizeGiB: 4 },
        { mount: '/var/tmp', sizeGiB: 2 },
        { mount: '/home', sizeGiB: 8 },
        { mount: '/var/lib/docker' },
      ],
    })
    expect(layoutError(SERVER)).toBe('')
  })

  it('simple sends nothing: the root fills the disk', () => {
    expect(toStorage(withPreset(SERVER, 'simple'))).toBeNull()
  })

  it('custom starts from what is shown', () => {
    const custom = withPreset(SERVER, 'custom')
    expect(custom.preset).toBe('custom')
    expect(toStorage(custom)).toEqual(toStorage(SERVER))
    expect(withPreset({ ...custom, root: '40' }, 'server').root).toBe('16')
  })

  it('needs the boot partition, the root, every volume and 1 GiB for the rest', () => {
    expect(neededBytes(SERVER)).toBe((16 + 4 + 2 + 8 + 1) * GIB + 274 * 1024 ** 2)
    expect(neededBytes(withPreset(SERVER, 'simple'))).toBe(0)
  })

  it('names the first problem', () => {
    const custom = withPreset(SERVER, 'custom')
    expect(layoutError({ ...custom, root: '3' })).toBe('The root needs at least 4 GiB.')
    expect(layoutError({ ...custom, root: '' })).toBe('The root needs at least 4 GiB.')
    expect(layoutError({ ...custom, tmp: '0' })).toBe('/tmp needs at least 1 GiB.')
    expect(layoutError({ ...custom, root: '5000' })).toBe('Sizes go up to 4096 GiB.')
    expect(layoutError({ ...custom, volumes: [] })).toBe(
      'Add at least one folder, or choose Simple.',
    )
    expect(layoutError({ ...custom, volumes: [{ mount: '/etc', size: '1', rest: false }] })).toBe(
      'Choose a folder from the list.',
    )
    const rows = custom.volumes
    expect(layoutError({ ...custom, volumes: [{ ...rows[0]!, size: '' }] })).toBe(
      'Give each folder a size, or let one take the rest of the disk.',
    )
    expect(layoutError({ ...custom, volumes: [rows[0]!, rows[0]!] })).toBe(
      'Each folder can have only one volume.',
    )
    expect(layoutError({ ...custom, volumes: [{ ...rows[0]!, rest: true }, rows[3]!] })).toBe(
      'Only one folder can take the rest of the disk.',
    )
  })

  it('adds a row for the next free folder, until none is left', () => {
    let layout = withPreset(SERVER, 'custom')
    layout = addRow(layout)
    expect(layout.volumes.at(-1)).toEqual({ mount: '/srv', size: '1', rest: false })
    layout = addRow(addRow(layout))
    expect(layout.volumes).toHaveLength(6)
    expect(addRow(layout).volumes).toHaveLength(6)
  })

  it('reads back what was saved, or starts simple', () => {
    expect(fromSettings({}).preset).toBe('simple')
    const saved = fromSettings({
      kuraDiskLayout: 'custom',
      kuraStorage: {
        rootGiB: 20,
        tmpGiB: 2,
        volumes: [{ mount: '/srv', sizeGiB: 50 }, { mount: '/home' }],
      },
    })
    expect(saved).toEqual({
      preset: 'custom',
      root: '20',
      tmp: '2',
      volumes: [
        { mount: '/srv', size: '50', rest: false },
        { mount: '/home', size: '', rest: true },
      ],
    })
    expect(fromSettings({ kuraDiskLayout: 'server', kuraStorage: 'junk' })).toEqual(SERVER)
    expect(fromSettings({ kuraDiskLayout: 'custom', kuraStorage: 'junk' }).preset).toBe('custom')
  })
})
