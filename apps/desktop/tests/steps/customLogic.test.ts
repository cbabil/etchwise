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

// Public key blobs whose first field names the key type (RFC 4253)
const ED25519 = 'AAAAC3NzaC1lZDI1NTE5AAAAIAABAgMEBQYHCAkKCwwNDg8QERITFBUWFxgZGhscHR4f'
const RSA =
  'AAAAB3NzaC1yc2EAAAADAQABAAAAQQABAgMEBQYHCAkKCwwNDg8QERITFBUWFxgZGhscHR4fICEiIyQlJicoKSorLC0uLzAxMjM0NTY3ODk6Ozw9Pj9A'

describe('PuTTY / RFC 4716 key files', () => {
  it('turns a multi-line block into one OpenSSH key, type read from the key data', () => {
    const file = [
      '---- BEGIN SSH2 PUBLIC KEY ----',
      'Comment: "chris@laptop"',
      ED25519.slice(0, 40),
      ED25519.slice(40),
      '---- END SSH2 PUBLIC KEY ----',
    ].join('\r\n')
    expect(splitKeys(file)).toEqual([`ssh-ed25519 ${ED25519} chris@laptop`])
  })

  it('handles RSA, a continued header, no comment, and plain keys around blocks', () => {
    const file = [
      'ssh-ed25519 AAAA plain@one',
      '---- BEGIN SSH2 PUBLIC KEY ----',
      'Subject: chris\\',
      ' continued',
      RSA.slice(0, 64),
      RSA.slice(64),
      '---- END SSH2 PUBLIC KEY ----',
      'ssh-rsa BBBB plain@two',
    ].join('\n')
    expect(splitKeys(file)).toEqual([
      'ssh-ed25519 AAAA plain@one',
      `ssh-rsa ${RSA}`,
      'ssh-rsa BBBB plain@two',
    ])
  })

  it('drops a block that never ends or holds no readable key, keeping the keys after it', () => {
    expect(splitKeys(['---- BEGIN SSH2 PUBLIC KEY ----', ED25519].join('\n'))).toEqual([])
    expect(
      splitKeys(
        ['---- BEGIN SSH2 PUBLIC KEY ----', ED25519, 'ssh-ed25519 AAAA after@block'].join('\n'),
      ),
    ).toEqual(['ssh-ed25519 AAAA after@block'])
    expect(
      splitKeys(
        ['---- BEGIN SSH2 PUBLIC KEY ----', '!!!', '---- END SSH2 PUBLIC KEY ----'].join('\n'),
      ),
    ).toEqual([])
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
