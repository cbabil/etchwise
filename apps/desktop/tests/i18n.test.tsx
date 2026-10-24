import { act, render, screen, waitFor } from '@testing-library/react'

import {
  APP_NAME,
  I18nProvider,
  substitute,
  translate,
  UPSTREAM_BRAND,
  useLanguage,
  useTr,
} from '@/i18n'
import type { Catalogue } from '@/i18n/overlay'

function Probe() {
  const tr = useTr('AppOptionsDialog')
  const trDevice = useTr('DeviceSelectionStep')
  const { setLanguage } = useLanguage()
  return (
    <>
      <button type="button" onClick={() => setLanguage('de')}>
        {tr('App Options')}
      </button>
      <h1>{trDevice('Select your device')}</h1>
    </>
  )
}

describe('i18n', () => {
  it('substitutes Qt placeholders', () => {
    expect(substitute('Imager %1', ['v1.0.0'])).toBe('Imager v1.0.0')
    expect(substitute('%2 of %1', ['a'])).toBe('%2 of a')
    expect(translate({ C: { Hi: 'Hallo %1' } }, 'C', 'Hi', ['x'])).toBe('Hallo x')
    expect(translate({}, 'C', 'Hi')).toBe('Hi')
  })

  it('shows the app name in place of the upstream brand', () => {
    expect(APP_NAME).toBe('Etchwise')
    expect(translate({}, 'C', 'Close Raspberry Pi Imager and exit the application')).toBe(
      'Close Etchwise and exit the application',
    )
    expect(translate({}, 'C', 'Raspberry Pi Imager %1', ['v1.0.0'])).toBe('Etchwise v1.0.0')
    expect(translate({}, 'C', 'Choose a Raspberry Pi device')).toBe('Choose a Raspberry Pi device')
  })

  it('renames the brand in every translation, whatever its spelling', () => {
    const catalogues = import.meta.glob<{ default: Catalogue }>('../src/i18n/locales/*.json', {
      eager: true,
    })
    const leftovers: string[] = []
    for (const [file, { default: catalogue }] of Object.entries(catalogues)) {
      for (const [context, entries] of Object.entries(catalogue)) {
        for (const source of Object.keys(entries)) {
          if (!source.includes('Raspberry Pi Imager')) continue
          const shown = translate(catalogue, context, source)
          // The swap's own pattern, plus the English name as a backstop.
          if (
            new RegExp(UPSTREAM_BRAND.source).test(shown) ||
            shown.includes('Raspberry Pi Imager')
          )
            leftovers.push(`${file}: ${shown}`)
        }
      }
    }
    expect(leftovers).toEqual([])
  })

  it('switches language from the upstream catalogues and the Etchwise overlay', async () => {
    render(
      <I18nProvider>
        <Probe />
      </I18nProvider>,
    )
    expect(screen.getByRole('button')).toHaveTextContent('App Options')
    act(() => screen.getByRole('button').click())
    await waitFor(() => expect(screen.getByRole('button')).toHaveTextContent('Einstellungen'))
    expect(screen.getByRole('heading')).toHaveTextContent('Wählen Sie Ihr Gerät aus')
  })
})
