import { fireEvent, screen, waitFor, within } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { LocaleCustomizationStep } from '@/wizard/steps/LocaleCustomizationStep'

import { answer, callsTo, nextButton, renderStep } from './customHarness'

vi.mock('@/api/imager', () => ({
  call: vi.fn(async () => null),
  onImagerEvent: vi.fn(async () => () => undefined),
}))

const lists = {
  getCapitalCitiesList: () => ['London (United Kingdom)', 'Paris (France)', 'Berlin (Germany)'],
  getTimezoneList: () => ['Europe/London', 'Europe/Paris', 'Europe/Berlin', 'America/New_York'],
  getKeymapLayoutList: () => ['gb', 'us', 'fr', 'de'],
  getLocaleDataForCapital: (city: unknown) =>
    city === 'Paris (France)'
      ? { timezone: 'Europe/Paris', keyboard: 'fr', countryCode: 'FR' }
      : city === 'Berlin (Germany)'
        ? { timezone: 'Europe/Berlin', keyboard: 'de', countryCode: 'DE' }
        : { timezone: 'Europe/London', keyboard: 'gb', countryCode: 'GB' },
}

const combo = (label: RegExp) => screen.getByRole('combobox', { name: label })

function pick(label: RegExp, option: string) {
  fireEvent.click(combo(label))
  fireEvent.click(within(screen.getByRole('listbox')).getByText(option))
}

describe('LocaleCustomizationStep', () => {
  it('defaults to the system time zone and gb keyboard for Europe/London', async () => {
    answer({ ...lists, getTimezone: () => 'Europe/London' })
    await renderStep(LocaleCustomizationStep)
    await waitFor(() => expect(combo(/^Time zone:/)).toHaveTextContent('Europe/London'))
    expect(combo(/^Keyboard layout:/)).toHaveTextContent('gb')
    expect(combo(/^Capital city:/)).toHaveTextContent('')
  })

  it('defaults to us elsewhere and restores saved values', async () => {
    answer({ ...lists, getTimezone: () => 'America/New_York' })
    await renderStep(LocaleCustomizationStep)
    await waitFor(() => expect(combo(/^Keyboard layout:/)).toHaveTextContent('us'))
  })

  it('restores a saved city and refreshes the recommended Wi-Fi country', async () => {
    answer({
      ...lists,
      getTimezone: () => 'Europe/London',
      getSavedCustomisationSettings: () => ({
        capitalCity: 'Paris (France)',
        timezone: 'Europe/Berlin',
        keyboard: 'de',
      }),
    })
    const { wizard } = await renderStep(LocaleCustomizationStep)
    await waitFor(() => expect(wizard().customizationSettings.recommendedWifiCountry).toBe('FR'))
    expect(combo(/^Capital city:/)).toHaveTextContent('Paris (France)')
    expect(combo(/^Time zone:/)).toHaveTextContent('Europe/Berlin')
    expect(combo(/^Keyboard layout:/)).toHaveTextContent('de')
    expect(callsTo('setPersistedCustomisationSetting')).toContainEqual([
      'recommendedWifiCountry',
      'FR',
    ])
  })

  it('pre-fills time zone and keyboard from the city unless changed by hand', async () => {
    answer({ ...lists, getTimezone: () => 'Europe/London' })
    await renderStep(LocaleCustomizationStep)
    await waitFor(() => expect(combo(/^Time zone:/)).toHaveTextContent('Europe/London'))
    pick(/^Capital city:/, 'Paris (France)')
    await waitFor(() => expect(combo(/^Time zone:/)).toHaveTextContent('Europe/Paris'))
    expect(combo(/^Keyboard layout:/)).toHaveTextContent('fr')
    pick(/^Keyboard layout:/, 'us')
    pick(/^Capital city:/, 'Berlin (Germany)')
    await waitFor(() => expect(combo(/^Time zone:/)).toHaveTextContent('Europe/Berlin'))
    expect(combo(/^Keyboard layout:/)).toHaveTextContent('us')
    pick(/^Time zone:/, 'America/New_York')
    pick(/^Capital city:/, 'London (United Kingdom)')
    await waitFor(() => expect(combo(/^Capital city:/)).toHaveTextContent('London'))
    expect(combo(/^Time zone:/)).toHaveTextContent('America/New_York')
  })

  it('saves all three values on Next and sets localeConfigured', async () => {
    answer({ ...lists, getTimezone: () => 'Europe/London' })
    const { wizard } = await renderStep(LocaleCustomizationStep, { currentStep: Step.Locale })
    await waitFor(() => expect(combo(/^Time zone:/)).toHaveTextContent('Europe/London'))
    pick(/^Capital city:/, 'Paris (France)')
    await waitFor(() => expect(combo(/^Time zone:/)).toHaveTextContent('Europe/Paris'))
    fireEvent.click(nextButton())
    expect(wizard()).toMatchObject({ localeConfigured: true, currentStep: Step.User })
    expect(wizard().customizationSettings).toMatchObject({
      capitalCity: 'Paris (France)',
      timezone: 'Europe/Paris',
      keyboard: 'fr',
    })
  })

  it('removes empty values and keeps an unlisted time zone as text', async () => {
    answer({
      ...lists,
      getTimezone: () => '',
      getSavedCustomisationSettings: () => ({ keyboard: 'xx' }),
    })
    const { wizard } = await renderStep(LocaleCustomizationStep, { currentStep: Step.Locale })
    await waitFor(() => expect(callsTo('getTimezone')).toHaveLength(1))
    await waitFor(() => expect(combo(/^Keyboard layout:/)).toHaveAccessibleName(/xx/))
    fireEvent.click(nextButton())
    expect(callsTo('removePersistedCustomisationSetting')).toEqual([['capitalCity'], ['timezone']])
    expect(wizard().customizationSettings).toMatchObject({ keyboard: 'xx' })
    expect(wizard().localeConfigured).toBe(true)
  })
})
