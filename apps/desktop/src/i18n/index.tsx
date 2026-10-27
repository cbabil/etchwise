/**
 * Translations in the shape of upstream's Qt catalogues: `qsTr("text")`
 * inside `Foo.qml` becomes `tr('Foo', 'text')`, with `%1`…`%n`
 * placeholders. English is the source text itself.
 */
import { createContext, type ReactNode, useCallback, useContext, useEffect, useState } from 'react'

import { type Catalogue, withOverlay } from './overlay'

const loaders = import.meta.glob<{ default: Catalogue }>('./locales/*.json')

interface I18n {
  language: string
  catalogue: Catalogue
  setLanguage: (language: string) => void
}

const I18nContext = createContext<I18n>({
  language: 'en',
  catalogue: {},
  setLanguage: () => undefined,
})

/** Replaces `%1`…`%9` with `args` (Qt `QString::arg`). */
export function substitute(text: string, args: readonly unknown[]): string {
  return text.replace(/%(\d)/g, (match, n: string) => {
    const value = args[Number(n) - 1]
    return value === undefined ? match : String(value)
  })
}

/** The app's own name, shown wherever upstream's text names its app. */
export const APP_NAME = 'Etchwise'

/**
 * How upstream's text and its translations name the app: English (kept
 * verbatim by most languages), a Georgian misspelling, Korean, and two
 * Hebrew phrasings ("the image burner of / for Raspberry Pi").
 */
export const UPSTREAM_BRAND =
  /Raspberry Pi Imager|Rasperry Pi Imager|라즈베리 파이 이미저|צורב הדמויות (?:של |ל־)Raspberry Pi/g

/**
 * Setup screens whose "Raspberry Pi" means the device being set up. When
 * the chosen device is not a Raspberry Pi, it is named instead there.
 * Elsewhere (repository, update, Connect, Secure Boot) the words name
 * Raspberry Pi's products and stay.
 */
const DEVICE_WORDING_CONTEXTS: ReadonlySet<string> = new Set([
  'OSSelectionStep',
  'HostnameCustomizationStep',
  'LocaleCustomizationStep',
  'UserCustomizationStep',
  'RemoteAccessStep',
])

/**
 * "Raspberry Pi" as the translations write it: as is (and the French
 * catalogue's "Rasbperry Pi" typo), Korean, Chinese; not in a product
 * name ("Raspberry Pi OS", "Raspberry Pi Connect").
 */
const DEVICE_WORD = /(?:Raspberry Pi|Rasbperry Pi|라즈베리 파이|树莓派)(?! (?:OS|Connect)\b)/g

/**
 * Languages whose grammar is bound to the name (Turkish suffixes, the
 * Hungarian accusative, Korean particles chosen for a word ending in a
 * vowel): they say "your device" with the matching ending instead of the
 * device's name.
 */
const DEVICE_NOUN: Readonly<Record<string, ReadonlyArray<readonly [RegExp, string]>>> = {
  tr: [
    [/Raspberry Pi'nizin/g, 'cihazınızın'],
    [/Raspberry Pi'nizi/g, 'cihazınızı'],
    [/Raspberry Pi'nize/g, 'cihazınıza'],
    [/Raspberry Pi'niz/g, 'cihazınız'],
    [/Raspberry Pi(?! (?:OS|Connect)\b)/g, 'cihaz'],
  ],
  hu: [[/a Raspberry Pi-t/g, 'az eszközt']],
  ko: [[/라즈베리 파이(?! (?:OS|Connect)\b)/g, '장치']],
}

/** `text` naming `device` (or "your device", see DEVICE_NOUN) in place of "Raspberry Pi". */
function nameDevice(text: string, device: string, language: string): string {
  let named = text
  for (const [word, noun] of DEVICE_NOUN[language] ?? []) named = named.replace(word, noun)
  named = named.replace(DEVICE_WORD, () => device)
  // A sentence that began with the name begins with a capital again
  return /^\p{Ll}/u.test(named) && !/^\p{Ll}/u.test(text)
    ? named.charAt(0).toLocaleUpperCase(language) + named.slice(1)
    : named
}

/** The chosen non-Pi device and the language, for the setup screens' wording. */
export interface TranslateOptions {
  device?: string
  language?: string
}

/**
 * Looks a source string up in a catalogue, naming this app as `APP_NAME`
 * and, on the setup screens, a non-Pi `device` (taken literally, never read
 * as a pattern). Arguments are substituted last, so they are never renamed.
 */
export function translate(
  catalogue: Catalogue,
  context: string,
  source: string,
  args: readonly unknown[] = [],
  { device = '', language = 'en' }: TranslateOptions = {},
): string {
  const text = (catalogue[context]?.[source] ?? source).replace(UPSTREAM_BRAND, APP_NAME)
  const named =
    device && DEVICE_WORDING_CONTEXTS.has(context) ? nameDevice(text, device, language) : text
  return substitute(named, args)
}

/** The chosen device's name when it is not a Raspberry Pi, else empty. */
const DeviceWordingContext = createContext('')

/** Provided by the wizard, which knows the chosen device. */
export function DeviceWordingProvider({
  device,
  children,
}: {
  device: string
  children: ReactNode
}) {
  return <DeviceWordingContext.Provider value={device}>{children}</DeviceWordingContext.Provider>
}

export function I18nProvider({
  initial = 'en',
  children,
}: {
  initial?: string
  children: ReactNode
}) {
  const [language, setLanguage] = useState(initial)
  const [loaded, setLoaded] = useState<Record<string, Catalogue>>({})
  useEffect(() => {
    const load = loaders[`./locales/${language}.json`]
    if (!load || loaded[language]) return
    let live = true
    void load().then((module) => {
      if (live) setLoaded((all) => ({ ...all, [language]: withOverlay(language, module.default) }))
    })
    return () => {
      live = false
    }
  }, [language, loaded])
  const catalogue = loaded[language] ?? {}
  return (
    <I18nContext.Provider value={{ language, catalogue, setLanguage }}>
      {children}
    </I18nContext.Provider>
  )
}

/** `qsTr` for one QML context: `const tr = useTr('WizardContainer')`. */
export function useTr(context: string): (source: string, ...args: unknown[]) => string {
  const { catalogue, language } = useContext(I18nContext)
  const chosen = useContext(DeviceWordingContext)
  // Only the setup screens' strings change with the device
  const device = DEVICE_WORDING_CONTEXTS.has(context) ? chosen : ''
  return useCallback(
    (source: string, ...args: unknown[]) =>
      translate(catalogue, context, source, args, { device, language }),
    [catalogue, context, device, language],
  )
}

export function useLanguage(): Pick<I18n, 'language' | 'setLanguage'> {
  const { language, setLanguage } = useContext(I18nContext)
  return { language, setLanguage }
}
