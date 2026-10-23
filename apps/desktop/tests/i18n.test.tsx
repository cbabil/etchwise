import { act, render, screen, waitFor } from '@testing-library/react'

import { I18nProvider, substitute, translate, useLanguage, useTr } from '@/i18n'

function Probe() {
  const tr = useTr('AppOptionsDialog')
  const { setLanguage } = useLanguage()
  return (
    <button type="button" onClick={() => setLanguage('de')}>
      {tr('App Options')}
    </button>
  )
}

describe('i18n', () => {
  it('substitutes Qt placeholders', () => {
    expect(substitute('Raspberry Pi Imager %1', ['v2.0.7'])).toBe('Raspberry Pi Imager v2.0.7')
    expect(substitute('%2 of %1', ['a'])).toBe('%2 of a')
    expect(translate({ C: { Hi: 'Hallo %1' } }, 'C', 'Hi', ['x'])).toBe('Hallo x')
    expect(translate({}, 'C', 'Hi')).toBe('Hi')
  })

  it('switches language from the upstream catalogues', async () => {
    render(
      <I18nProvider>
        <Probe />
      </I18nProvider>,
    )
    expect(screen.getByRole('button')).toHaveTextContent('App Options')
    act(() => screen.getByRole('button').click())
    await waitFor(() => expect(screen.getByRole('button')).toHaveTextContent('Einstellungen'))
  })
})
