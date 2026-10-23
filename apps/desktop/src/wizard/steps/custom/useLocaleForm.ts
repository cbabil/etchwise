/**
 * State of upstream `LocaleCustomizationStep.qml`: the three lists, the
 * selected city / time zone / keyboard (kept as text, like the combo's
 * `editText`, so a value missing from its list survives), and the capital
 * city auto-fill that respects fields the user changed by hand.
 */
import { useRef, useState } from 'react'

import { call } from '@/api/imager'

import { useWizard, type WizardApi } from '../../WizardContext'
import { editSettings, persist, str, unpersist } from './settings'
import { useMountEffect } from './useMountEffect'

interface LocaleData {
  timezone?: string
  keyboard?: string
  countryCode?: string
}

export interface LocaleLists {
  cities: readonly string[]
  timezones: readonly string[]
  keymaps: readonly string[]
}

const asList = (value: unknown): string[] =>
  Array.isArray(value) ? value.map((v) => String(v)) : []

async function loadLists(): Promise<LocaleLists & { systemTz: string }> {
  const [cities, timezones, keymaps, systemTz] = await Promise.all([
    call('getCapitalCitiesList'),
    call('getTimezoneList'),
    call('getKeymapLayoutList'),
    call('getTimezone'),
  ])
  return {
    cities: asList(cities),
    timezones: asList(timezones),
    keymaps: asList(keymaps),
    systemTz: str(systemTz),
  }
}

/** Saves the recommended Wi-Fi country of `data` (upstream `recommendedWifiCountry`). */
function saveCountry(wizard: WizardApi, data: LocaleData): void {
  if (!data.countryCode) return
  editSettings(wizard, { recommendedWifiCountry: data.countryCode })
  persist('recommendedWifiCountry', data.countryCode)
}

/** Upstream `onNextClicked`: stores or removes each value and sets `localeConfigured`. */
export function saveLocale(wizard: WizardApi, values: Record<string, string>): void {
  Object.entries(values).forEach(([key, raw]) => {
    const value = raw.trim()
    if (value.length > 0) {
      editSettings(wizard, { [key]: value })
      persist(key, value)
    } else {
      editSettings(wizard, {}, [key])
      unpersist(key)
    }
  })
  const configured = (values.timezone ?? '').trim().length > 0
  wizard.update({ localeConfigured: configured || (values.keyboard ?? '').trim().length > 0 })
}

export function useLocaleForm() {
  const wizard = useWizard()
  const [lists, setLists] = useState<LocaleLists>({ cities: [], timezones: [], keymaps: [] })
  const [city, setCity] = useState('')
  const [timezone, setTimezone] = useState('')
  const [keyboard, setKeyboard] = useState('')
  const changed = useRef({ timezone: false, keyboard: false })

  const fetchCapital = async (name: string): Promise<LocaleData> =>
    (await call<LocaleData | null>('getLocaleDataForCapital', name)) ?? {}

  useMountEffect(() => {
    const settings = wizard.customizationSettings
    void loadLists().then(async ({ systemTz, ...loaded }) => {
      setLists(loaded)
      const savedCity = str(settings.capitalCity)
      if (savedCity && loaded.cities.includes(savedCity)) {
        setCity(savedCity)
        saveCountry(wizard, await fetchCapital(savedCity))
      }
      const tz = str(settings.timezone) || systemTz
      setTimezone(tz)
      setKeyboard(str(settings.keyboard) || (tz === 'Europe/London' ? 'gb' : 'us'))
    })
  })

  const pickCity = async (name: string) => {
    setCity(name)
    if (!name) return
    const data = await fetchCapital(name)
    if (!changed.current.timezone && data.timezone) setTimezone(data.timezone)
    if (!changed.current.keyboard && data.keyboard) setKeyboard(data.keyboard)
    saveCountry(wizard, data)
  }
  const pickTimezone = (value: string) => {
    changed.current.timezone = true
    setTimezone(value)
  }
  const pickKeyboard = (value: string) => {
    changed.current.keyboard = true
    setKeyboard(value)
  }
  const save = () => saveLocale(wizard, { capitalCity: city, timezone, keyboard })
  return { lists, city, timezone, keyboard, pickCity, pickTimezone, pickKeyboard, save }
}
