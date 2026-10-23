import { fireEvent, screen, waitFor } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { WifiCustomizationStep } from '@/wizard/steps/WifiCustomizationStep'

import { answer, callsTo, nextButton, renderStep } from './customHarness'

vi.mock('@/api/imager', () => ({
  call: vi.fn(async () => null),
  onImagerEvent: vi.fn(async () => () => undefined),
}))

const ssid = () => screen.getByLabelText('SSID:')
const password = () => screen.getByLabelText('Password:')
const confirm = () => screen.getByLabelText('Confirm password:')
const type = (el: HTMLElement, value: string) => fireEvent.change(el, { target: { value } })
const tab = (name: RegExp) => screen.getByRole('button', { name })
let saved: Record<string, unknown> = {}

const backend = (extra: Record<string, (...args: unknown[]) => unknown> = {}) =>
  answer({
    getSavedCustomisationSettings: () => ({ ...saved }),
    setSavedCustomisationSettings: (map) => void (saved = map as Record<string, unknown>),
    pbkdf2: (pw, s) => `pbkdf2(${String(pw)},${String(s)})`,
    ...extra,
  })

function setUserAgent(value: string) {
  Object.defineProperty(navigator, 'userAgent', { value, configurable: true })
}

describe('WifiCustomizationStep', () => {
  beforeEach(() => {
    saved = {}
    setUserAgent('Linux')
  })

  it('auto-detects the SSID and prefills the PSK without consent off macOS', async () => {
    backend({ getSSID: () => 'HomeNet', getPSKForSSID: () => 'hunter22' })
    await renderStep(WifiCustomizationStep)
    await waitFor(() => expect(password()).toHaveValue('hunter22'))
    expect(ssid()).toHaveValue('HomeNet')
    expect(confirm()).toHaveValue('hunter22')
    expect(callsTo('getPSKForSSID')).toEqual([['HomeNet']])
    expect(tab(/^Secure network/)).toHaveAttribute('aria-pressed', 'true')
  })

  it('asks for keychain consent on macOS and respects No / Yes', async () => {
    setUserAgent('Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)')
    backend({ getSSID: () => 'HomeNet', getPSKForSSID: () => 'hunter22' })
    await renderStep(WifiCustomizationStep)
    await screen.findByRole('dialog', { name: 'Keychain Access' })
    fireEvent.click(screen.getByRole('button', { name: /^No,/ }))
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
    expect(callsTo('getPSKForSSID')).toEqual([])
  })

  it('reads the keychain after Yes on macOS', async () => {
    setUserAgent('Macintosh')
    backend({ getSSID: () => 'HomeNet', getPSKForSSID: () => 'hunter22' })
    await renderStep(WifiCustomizationStep)
    fireEvent.click(await screen.findByRole('button', { name: /^Yes,/ }))
    await waitFor(() => expect(password()).toHaveValue('hunter22'))
  })

  it('shows inline errors and gates Next', async () => {
    backend()
    await renderStep(WifiCustomizationStep)
    expect(nextButton()).toBeEnabled()
    type(ssid(), 'Net')
    expect(screen.getByRole('alert')).toHaveTextContent('Enter a password')
    expect(nextButton()).toBeDisabled()
    type(password(), 'short')
    expect(screen.getByRole('alert')).toHaveTextContent('Password is too short (min 8 characters)')
    type(password(), 'x'.repeat(64))
    expect(screen.getByRole('alert')).toHaveTextContent('Password is too long (max 63 characters)')
    type(password(), 'café-latte')
    expect(screen.getByRole('alert')).toHaveTextContent('Password contains unsupported characters')
    type(password(), 'goodpassword')
    expect(screen.getByRole('alert')).toHaveTextContent("Passwords don't match")
    type(confirm(), 'goodpassword')
    expect(screen.queryByRole('alert')).toBeNull()
    expect(nextButton()).toBeEnabled()
  })

  it('hashes a passphrase with PBKDF2 and saves everything on Next', async () => {
    backend()
    const { wizard } = await renderStep(WifiCustomizationStep, { currentStep: Step.Wifi })
    type(ssid(), ' Net ')
    type(password(), 'goodpassword')
    type(confirm(), 'goodpassword')
    fireEvent.click(screen.getByRole('checkbox', { name: 'Hidden SSID' }))
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.RemoteAccess))
    const expected = {
      wifiMode: 'secure',
      wifiSSID: 'Net',
      wifiPasswordCrypt: 'pbkdf2(goodpassword,Net)',
      wifiHidden: true,
    }
    expect(wizard().customizationSettings).toMatchObject(expected)
    expect(wizard().wifiConfigured).toBe(true)
    expect(saved).toEqual(expected)
  })

  it('keeps a 64-hex key as-is', async () => {
    backend()
    const { wizard } = await renderStep(WifiCustomizationStep, { currentStep: Step.Wifi })
    const hex = 'ab'.repeat(32)
    type(ssid(), 'Net')
    type(password(), hex)
    type(confirm(), hex)
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().customizationSettings.wifiPasswordCrypt).toBe(hex))
    expect(callsTo('pbkdf2')).toEqual([])
  })

  it('keeps the saved crypt for an unchanged SSID and drops it when the SSID changes', async () => {
    saved = { wifiSSID: 'Net', wifiPasswordCrypt: 'savedcrypt', wifiHidden: 'true' }
    backend()
    const { wizard } = await renderStep(WifiCustomizationStep, { currentStep: Step.Wifi })
    expect(screen.getByRole('checkbox', { name: 'Hidden SSID' })).toBeChecked()
    expect(password()).toHaveAttribute('placeholder', 'Saved (hidden) — leave blank to keep')
    expect(nextButton()).toBeEnabled()
    type(ssid(), 'Other')
    expect(password()).toHaveAttribute('placeholder', 'Network password')
    expect(nextButton()).toBeDisabled()
    type(ssid(), 'Net')
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.RemoteAccess))
    expect(wizard().customizationSettings.wifiPasswordCrypt).toBe('savedcrypt')
    expect(saved.wifiPasswordCrypt).toBe('savedcrypt')
    expect(callsTo('getSSID')).toEqual([])
    expect(callsTo('getPSKForSSID')).toEqual([])
  })

  it('open mode hides and clears the password and removes the crypt', async () => {
    saved = { wifiSSID: 'Cafe', wifiPasswordCrypt: 'old', wifiMode: 'secure' }
    backend()
    const { wizard } = await renderStep(WifiCustomizationStep, { currentStep: Step.Wifi })
    type(password(), 'typed-secret')
    fireEvent.click(tab(/^Open network/))
    expect(screen.queryByLabelText('Password:')).toBeNull()
    fireEvent.click(tab(/^Secure network/))
    expect(password()).toHaveValue('')
    fireEvent.click(tab(/^Open network/))
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.RemoteAccess))
    expect(wizard().customizationSettings).toMatchObject({ wifiMode: 'open', wifiSSID: 'Cafe' })
    expect(wizard().customizationSettings).not.toHaveProperty('wifiPasswordCrypt')
    expect(saved).toEqual({ wifiMode: 'open', wifiSSID: 'Cafe', wifiHidden: false })
  })

  it('an empty SSID clears Wi-Fi settings', async () => {
    saved = { wifiSSID: 'Net', wifiPasswordCrypt: 'c', wifiHidden: false, hostname: 'pi' }
    backend()
    const { wizard } = await renderStep(WifiCustomizationStep, {
      currentStep: Step.Wifi,
      wifiConfigured: true,
    })
    type(ssid(), '')
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.RemoteAccess))
    expect(wizard().wifiConfigured).toBe(false)
    expect(saved).toEqual({ hostname: 'pi', wifiMode: 'secure' })
  })
})
