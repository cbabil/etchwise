import { act, fireEvent, screen, waitFor, within } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { resultOf } from '@/wizard/steps/custom2/ifFeatures'
import { USB_GADGET_HELP_URL } from '@/wizard/steps/custom2/IfFeaturesSections'
import { IfAndFeaturesCustomizationStep } from '@/wizard/steps/IfAndFeaturesCustomizationStep'

import { callsOf, setAnswers } from '../dialogs/imagerMock'
import { renderInWizard } from '../dialogs/renderWizard'

vi.mock('@/api/imager', () => import('../dialogs/imagerMock'))

const allCaps = {
  i2c: true,
  spi: true,
  onewire: true,
  serial: true,
  serialConsoleOnly: true,
  usbOtg: true,
}

function backend(caps: Partial<typeof allCaps>, saved: Record<string, unknown> = {}) {
  const map: Record<string, boolean | undefined> = {
    i2c: caps.i2c,
    spi: caps.spi,
    onewire: caps.onewire,
    serial: caps.serial,
    usb_otg: caps.usbOtg,
  }
  setAnswers({
    checkHWAndSWCapability: (cap) => map[String(cap)] ?? false,
    checkHWCapability: () => caps.serialConsoleOnly ?? false,
    getSavedCustomisationSettings: () => saved,
  })
}

const renderStep = (initial = {}) =>
  renderInWizard(<IfAndFeaturesCustomizationStep />, {
    step: Step.IfAndFeatures,
    initial: { ccRpiAvailable: true, ifAndFeaturesAvailable: true, ...initial },
  })

const pill = (name: string) => screen.getByRole('switch', { name: new RegExp(`^${name}`) })
const next = () => fireEvent.click(screen.getByRole('button', { name: /^Next/ }))

afterEach(() => vi.useRealTimers())

describe('IfAndFeaturesCustomizationStep', () => {
  it('moves on at once when the OS has no cc-rpi support', async () => {
    backend(allCaps)
    const { wizard } = await renderStep({ ccRpiAvailable: false })
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Writing))
    expect(wizard().ifAndFeaturesAvailable).toBe(false)
  })

  it('moves on when the device supports none of the options', async () => {
    backend({})
    const { wizard } = await renderStep()
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Writing))
    expect(wizard().ifSerial).toBe('Disabled')
  })

  it('shows only supported options, prefilled, and saves them on Next', async () => {
    backend(allCaps, { enableI2C: 'true', enableSerial: 'Console' })
    const { wizard } = await renderStep()
    await screen.findByRole('heading', { name: 'Interfaces' })
    expect(pill('Enable I2C')).toBeChecked()
    expect(pill('Enable SPI')).not.toBeChecked()
    expect(screen.getByRole('combobox')).toHaveTextContent('Console')
    fireEvent.click(pill('Enable SPI'))
    fireEvent.click(pill('Enable 1-Wire'))
    fireEvent.click(screen.getByRole('combobox'))
    fireEvent.click(within(screen.getByRole('listbox')).getByText('Hardware'))
    next()
    expect(wizard().customizationSettings).toMatchObject({
      enableI2C: true,
      enableSPI: true,
      enable1Wire: true,
      enableSerial: 'Hardware',
      enableUsbGadget: false,
    })
    expect(wizard()).toMatchObject({ ifI2cEnabled: true, ifSerial: 'Hardware' })
    expect(callsOf('removePersistedCustomisationSetting')).toHaveLength(5)
    expect(wizard().currentStep).toBe(Step.Writing)
  })

  it('hides the Interfaces section when only USB gadget mode is supported', async () => {
    backend({ usbOtg: true })
    await renderStep()
    await screen.findByRole('heading', { name: 'Features' })
    expect(screen.queryByRole('heading', { name: 'Interfaces' })).toBeNull()
    expect(screen.queryByRole('combobox')).toBeNull()
  })

  it('asks before enabling USB gadget mode; accept unlocks after 2 s', async () => {
    backend({ usbOtg: true })
    const { wizard } = await renderStep()
    await screen.findByRole('heading', { name: 'Features' })
    fireEvent.click(pill('Enable USB Gadget Mode'))
    vi.useFakeTimers()
    next()
    const dialog = screen.getByRole('dialog', { name: 'USB Gadget Mode Warning' })
    const accept = within(dialog).getByRole('button', { name: /^I understand, continue/ })
    expect(accept).toBeDisabled()
    fireEvent.click(within(dialog).getByRole('link', { name: 'documentation' }))
    expect(callsOf('openUrl')).toEqual([[USB_GADGET_HELP_URL]])
    fireEvent.click(within(dialog).getByRole('button', { name: /^Cancel/ }))
    act(() => vi.advanceTimersByTime(500))
    expect(wizard().currentStep).toBe(Step.IfAndFeatures)
    next()
    act(() => vi.advanceTimersByTime(2000))
    fireEvent.click(screen.getByRole('button', { name: /^I understand, continue/ }))
    expect(wizard().currentStep).toBe(Step.Writing)
    expect(wizard().featUsbGadgetEnabled).toBe(true)
  })

  it('does not ask when warnings are disabled, and skip clears the flags', async () => {
    backend({ usbOtg: true })
    const { wizard } = await renderStep({ disableWarnings: true })
    await screen.findByRole('heading', { name: 'Features' })
    fireEvent.click(pill('Enable USB Gadget Mode'))
    act(() => wizard().update({ disableWarnings: true }))
    next()
    expect(screen.queryByRole('dialog')).toBeNull()
    expect(wizard().currentStep).toBe(Step.Writing)
    act(() => wizard().jumpTo(Step.IfAndFeatures))
    fireEvent.click(await screen.findByRole('button', { name: /^Skip customisation/ }))
    expect(wizard()).toMatchObject({ featUsbGadgetEnabled: false, ifSerial: 'Disabled' })
  })

  it('maps Console to Default where console-only serial is unsupported', () => {
    const caps = { ...allCaps, serialConsoleOnly: false }
    const values = { i2c: false, spi: false, onewire: false, serialIndex: 4, usbGadget: false }
    expect(resultOf(caps, values).settings.enableSerial).toBe('Disabled')
    const consoleOnly = resultOf(allCaps, values)
    expect(consoleOnly.settings.enableSerial).toBe('Console')
    expect(resultOf({ ...allCaps, serial: false }, values).flags.ifSerial).toBe('Disabled')
  })
})
