/**
 * Translations in the shape of upstream's Qt catalogues: `qsTr("text")`
 * inside `Foo.qml` becomes `tr('Foo', 'text')`, with `%1`…`%n`
 * placeholders. English is the source text itself.
 */
import { createContext, type ReactNode, useCallback, useContext, useEffect, useState } from 'react'

type Catalogue = Record<string, Record<string, string>>

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
const UPSTREAM_BRAND =
  /Raspberry Pi Imager|Rasperry Pi Imager|라즈베리 파이 이미저|צורב הדמויות (?:של |ל־)Raspberry Pi/g

/** Looks a source string up in a catalogue, naming this app as `APP_NAME`. */
export function translate(
  catalogue: Catalogue,
  context: string,
  source: string,
  args: readonly unknown[] = [],
): string {
  const text = (catalogue[context]?.[source] ?? source).replace(UPSTREAM_BRAND, APP_NAME)
  return substitute(text, args)
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
      if (live) setLoaded((all) => ({ ...all, [language]: module.default }))
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
  const { catalogue } = useContext(I18nContext)
  return useCallback(
    (source: string, ...args: unknown[]) => translate(catalogue, context, source, args),
    [catalogue, context],
  )
}

export function useLanguage(): Pick<I18n, 'language' | 'setLanguage'> {
  const { language, setLanguage } = useContext(I18nContext)
  return { language, setLanguage }
}
