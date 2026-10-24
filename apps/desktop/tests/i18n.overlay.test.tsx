import { OVERLAY, withOverlay } from '@/i18n/overlay'
import { translations } from '@/wizard/steps/LanguageSelectionStep'

describe('Etchwise translation overlay', () => {
  it('translates every overlay string in every non-English language the app offers', () => {
    const languages = translations()
      .map((t) => t.code)
      .filter((code) => !code.startsWith('en'))
    expect(languages.length).toBeGreaterThan(0)
    const sources = Object.values(OVERLAY).flatMap((catalogue) =>
      Object.entries(catalogue).flatMap(([context, entries]) =>
        Object.keys(entries).map((source) => [context, source] as const),
      ),
    )
    const missing = languages.flatMap((lang) =>
      sources
        .filter(([context, source]) => !OVERLAY[lang]?.[context]?.[source])
        .map(([context, source]) => `${lang}: ${context} / ${source}`),
    )
    expect([...new Set(missing)]).toEqual([])
  })

  it('adds to the upstream catalogue without losing its entries', () => {
    const merged = withOverlay('de', { DeviceSelectionStep: { Next: 'Weiter' }, Other: { a: 'b' } })
    expect(merged.DeviceSelectionStep).toMatchObject({
      Next: 'Weiter',
      'Select your device': 'Wählen Sie Ihr Gerät aus',
    })
    expect(merged.Other).toEqual({ a: 'b' })
    expect(withOverlay('xx', { C: { a: 'b' } })).toEqual({ C: { a: 'b' } })
  })
})
