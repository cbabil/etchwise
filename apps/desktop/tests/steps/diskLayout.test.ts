import {
  addRow,
  fromSettings,
  type Layout,
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
      '/etc stays on the system disk.',
    )
    expect(layoutError({ ...custom, volumes: [{ mount: 'srv', size: '1', rest: false }] })).toBe(
      'Use a folder such as /srv/media.',
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

  it('adds a row for the next unused suggestion, then empty rows', () => {
    let layout = withPreset(SERVER, 'custom')
    layout = addRow(layout)
    expect(layout.volumes.at(-1)).toEqual({ mount: '/var', size: '1', rest: false })
    for (let i = 0; i < 5; i++) layout = addRow(layout)
    expect(layout.volumes).toHaveLength(10)
    expect(layout.volumes.map((v) => v.mount)).toEqual(
      expect.arrayContaining(['/var/lib', '/var/cache', '/tmp', '/srv', '/opt']),
    )
    layout = addRow(layout)
    expect(layout.volumes.at(-1)).toEqual({ mount: '', size: '1', rest: false })
    expect(addRow(layout).volumes).toHaveLength(12)
  })

  const custom = (volumes: [string, string][], extra: Partial<Layout> = {}): Layout => ({
    preset: 'custom',
    root: '16',
    tmp: '1',
    swap: '',
    volumes: volumes.map(([mount, size]) => ({ mount, size, rest: size === '' })),
    ...extra,
  })

  it('two folders cannot share a volume name, nor /swap the swap', () => {
    expect(
      layoutError(
        custom([
          ['/a/b', '4'],
          ['/a-b', '4'],
        ]),
      ),
    ).toBe('/a/b and /a-b would share a volume name.')
    expect(layoutError(custom([['/swap', '4']]))).toBe('')
    expect(layoutError(custom([['/swap', '4']], { swap: '2' }))).toBe(
      "/swap would share the swap volume's name.",
    )
  })

  it('swap: empty for none, whole GiB up to 4096, enough on its own', () => {
    expect(layoutError(custom([['/home', '']], { swap: '8' }))).toBe('')
    expect(layoutError(custom([], { swap: '8' }))).toBe('')
    expect(layoutError(custom([['/home', '']], { swap: '0' }))).toBe(
      'Swap needs at least 1 GiB, or leave it empty.',
    )
    expect(layoutError(custom([['/home', '']], { swap: '5000' }))).toBe('Sizes go up to 4096 GiB.')
    expect(toStorage(custom([['/home', '']], { swap: '8' }))).toEqual({
      rootGiB: 16,
      tmpGiB: 1,
      swapGiB: 8,
      volumes: [{ mount: '/home' }],
    })
    expect(toStorage(custom([['/home', '']]))).not.toHaveProperty('swapGiB')
    expect(neededBytes(custom([['/home', '4']], { swap: '8' }))).toBe(
      (16 + 4 + 8) * GIB + 274 * 1024 ** 2,
    )
  })

  it('/tmp is either in RAM or a volume', () => {
    const onDisk = custom([['/tmp', '4']], { tmp: '0' })
    expect(layoutError(onDisk)).toBe('')
    expect(toStorage(onDisk)).toEqual({ rootGiB: 16, volumes: [{ mount: '/tmp', sizeGiB: 4 }] })
    expect(layoutError(custom([['/home', '4']], { tmp: '0' }))).toBe('/tmp needs at least 1 GiB.')
  })

  it('the full layout: what Kura receives, and it fits a 256 GB drive', () => {
    const full = custom(
      [
        ['/var', '8'],
        ['/var/lib', '10'],
        ['/var/lib/docker', '40'],
        ['/var/log', '4'],
        ['/var/cache', '8'],
        ['/tmp', '4'],
        ['/opt', '10'],
        ['/home', '40'],
        ['/srv', '50'],
        ['/echange', '20'],
      ],
      { root: '20', swap: '8' },
    )
    expect(layoutError(full)).toBe('')
    expect(toStorage(full)).toEqual({
      rootGiB: 20,
      swapGiB: 8,
      volumes: [
        { mount: '/var', sizeGiB: 8 },
        { mount: '/var/lib', sizeGiB: 10 },
        { mount: '/var/lib/docker', sizeGiB: 40 },
        { mount: '/var/log', sizeGiB: 4 },
        { mount: '/var/cache', sizeGiB: 8 },
        { mount: '/tmp', sizeGiB: 4 },
        { mount: '/opt', sizeGiB: 10 },
        { mount: '/home', sizeGiB: 40 },
        { mount: '/srv', sizeGiB: 50 },
        { mount: '/echange', sizeGiB: 20 },
      ],
    })
    expect(neededBytes(full)).toBe(222 * GIB + 274 * 1024 ** 2)
    expect(neededBytes(full)).toBeLessThanOrEqual(256060514304)
  })

  it('reads back what was saved, or starts simple', () => {
    expect(fromSettings({}).preset).toBe('simple')
    const saved = fromSettings({
      kuraDiskLayout: 'custom',
      kuraStorage: {
        rootGiB: 20,
        tmpGiB: 2,
        swapGiB: 4,
        volumes: [{ mount: '/srv', sizeGiB: 50 }, { mount: '/home' }],
      },
    })
    expect(saved).toEqual({
      preset: 'custom',
      root: '20',
      tmp: '2',
      swap: '4',
      volumes: [
        { mount: '/srv', size: '50', rest: false },
        { mount: '/home', size: '', rest: true },
      ],
    })
    expect(fromSettings({ kuraDiskLayout: 'server', kuraStorage: 'junk' })).toEqual(SERVER)
    expect(fromSettings({ kuraDiskLayout: 'custom', kuraStorage: 'junk' })).toMatchObject({
      preset: 'custom',
      swap: '',
    })
  })
})
