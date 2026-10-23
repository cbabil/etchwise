import { call } from '@/api/imager'
import { browseKeyFile } from '@/wizard/steps/custom/browseKeyFile'
import { addKey, dedupeKeys, displayKey, savedKeys, splitKeys } from '@/wizard/steps/custom/sshKeys'
import {
  isValidWifiPassword,
  wifiNextEnabled,
  wifiPasswordError,
} from '@/wizard/steps/custom/validation'

vi.mock('@/api/imager', () => ({ call: vi.fn(async () => 'ssh-ed25519 AAA a\nssh-rsa BBB') }))
const open = vi.fn()
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: (...args: unknown[]) => open(...args) }))

const tr = (s: string) => s

describe('ssh key helpers', () => {
  it('splits, dedupes, adds and shortens keys', () => {
    expect(splitKeys('')).toEqual([])
    expect(dedupeKeys(['a', 'b', 'a'])).toEqual(['a', 'b'])
    expect(addKey(['a'], '  ')).toEqual(['a'])
    expect(addKey(['a'], ' a ')).toEqual(['a'])
    expect(displayKey('x'.repeat(50))).toBe(`${'x'.repeat(15)}...${'x'.repeat(15)}`)
    expect(displayKey('short')).toBe('short')
    expect(displayKey('ssh-rsa AAAA')).toBe('ssh-rsa AAAA')
    expect(savedKeys({ sshAuthorizedKeys: '', sshPublicKey: 5 })).toEqual([])
  })
})

describe('wifi validation', () => {
  const base = { mode: 'secure' as const, confirm: '', canKeep: false }
  it('accepts open, 64-hex and printable passphrases', () => {
    expect(isValidWifiPassword('')).toBe(true)
    expect(isValidWifiPassword('a'.repeat(64))).toBe(true)
    expect(isValidWifiPassword('cafécafé')).toBe(false)
    expect(wifiPasswordError({ ...base, mode: 'open', password: 'x' })).toBeNull()
    expect(
      wifiPasswordError({ ...base, password: 'a'.repeat(64), confirm: 'a'.repeat(64) }),
    ).toBeNull()
    expect(wifiNextEnabled('Net', { ...base, mode: 'open', password: '' })).toBe(true)
    expect(wifiNextEnabled('Net', { ...base, password: 'short', confirm: 'short' })).toBe(false)
  })
})

describe('browseKeyFile', () => {
  afterEach(() => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__
  })

  it('returns nothing outside Tauri', async () => {
    expect(await browseKeyFile(tr, tr)).toBe('')
    expect(open).not.toHaveBeenCalled()
  })

  it('opens the native dialog and reads the picked file', async () => {
    ;(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {}
    open.mockResolvedValueOnce('/home/pi/.ssh/id.pub')
    expect(await browseKeyFile(tr, tr)).toContain('ssh-rsa BBB')
    expect(call).toHaveBeenCalledWith('readFileContents', '/home/pi/.ssh/id.pub')
    open.mockResolvedValueOnce(null)
    expect(await browseKeyFile(tr, tr)).toBe('')
    open.mockResolvedValueOnce('/bad')
    vi.mocked(call).mockRejectedValueOnce(new Error('unreadable'))
    expect(await browseKeyFile(tr, tr)).toBe('')
  })
})
