import { pickFile } from '@/wizard/dialogs/common/pickFile'

const open = vi.fn()
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: (...args: unknown[]) => open(...args) }))
vi.mock('@tauri-apps/api/path', () => ({
  homeDir: async () => '/home/pi',
  join: async (...parts: string[]) => parts.join('/'),
}))

const options = { title: 'Pick', filters: [{ name: 'All', extensions: ['*'] }] }

describe('pickFile', () => {
  afterEach(() => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__
  })

  it('answers null outside Tauri', async () => {
    expect(await pickFile(options)).toBeNull()
    expect(open).not.toHaveBeenCalled()
  })

  it('opens the native dialog in the home sub-folder inside Tauri', async () => {
    ;(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {}
    open.mockResolvedValueOnce('/home/pi/.ssh/id.pem').mockResolvedValueOnce(null)
    expect(await pickFile({ ...options, homeSubdir: '.ssh' })).toBe('/home/pi/.ssh/id.pem')
    expect(open).toHaveBeenCalledWith(
      expect.objectContaining({ defaultPath: '/home/pi/.ssh', title: 'Pick' }),
    )
    expect(await pickFile(options)).toBeNull()
    expect(open.mock.calls[1]?.[0]).not.toHaveProperty('defaultPath')
  })
})
