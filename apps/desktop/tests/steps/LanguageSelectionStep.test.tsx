import { act, fireEvent, screen, waitFor } from '@testing-library/react'

import { I18nProvider } from '@/i18n'
import { Step } from '@/wizard/stepIds'
import { LanguageSelectionStep, translations } from '@/wizard/steps/LanguageSelectionStep'

import { answers, resetAnswers } from './selectionStepImager'
import { callsOf, renderStep } from './stepTestBackend'

vi.mock('@/api/imager', async () => (await import('./selectionStepImager')).fakeImager())

beforeEach(() => resetAnswers())

const combo = () => screen.findByRole('combobox', { name: /^Language:/ })

describe('translations', () => {
  it('lists every bundled catalogue by native name, sorted case-insensitively', () => {
    const list = translations()
    expect(list.find((l) => l.code === 'en')?.name).toBe('English')
    expect(list.find((l) => l.code === 'de')?.name).toBe('Deutsch')
    expect(list.find((l) => l.code === 'zh-TW')?.name).toBe('繁體中文')
    const names = list.map((l) => l.name.toLowerCase())
    expect([...names].sort()).toEqual(names)
  })
})

describe('LanguageSelectionStep', () => {
  it('pre-selects English (British) and goes to Device on Next, saving the language', async () => {
    const { wizard } = renderStep(<LanguageSelectionStep />)
    await waitFor(async () => expect(await combo()).toHaveTextContent('English (British)'))
    expect(screen.getByRole('heading', { level: 1, name: 'Welcome' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Back' })).toBeNull()
    fireEvent.click(screen.getByRole('button', { name: 'Next' }))
    expect(await callsOf('setSetting')).toContainEqual(['savedLanguage', 'English'])
    expect(await callsOf('changeLanguage')).toContainEqual(['English'])
    expect(wizard().currentStep).toBe(Step.Device)
  })

  it('pre-selects the backend language and goes to OS while offline', async () => {
    answers.getCurrentLanguage = () => 'Deutsch'
    answers.isOsListUnavailable = () => true
    const { wizard } = renderStep(
      <I18nProvider>
        <LanguageSelectionStep />
      </I18nProvider>,
    )
    await waitFor(async () => expect(await combo()).toHaveTextContent('Deutsch'))
    await waitFor(() => expect(wizard().hasNetworkConnectivity).toBe(false))
    fireEvent.click(screen.getByRole('button', { name: 'Next' }))
    expect(wizard().currentStep).toBe(Step.OS)
  })

  it('switches the interface language as soon as one is chosen', async () => {
    renderStep(
      <I18nProvider>
        <LanguageSelectionStep />
      </I18nProvider>,
    )
    const box = await combo()
    await waitFor(() => expect(box).toHaveTextContent('English (British)'))
    const index = translations().findIndex((l) => l.code === 'de')
    const english = translations().findIndex((l) => l.code === 'en')
    act(() => {
      for (let i = english; i < index; i++) fireEvent.keyDown(box, { key: 'ArrowDown' })
      for (let i = english; i > index; i--) fireEvent.keyDown(box, { key: 'ArrowUp' })
    })
    expect(await callsOf('changeLanguage')).toContainEqual(['Deutsch'])
    await screen.findByRole('heading', { level: 1, name: 'Willkommen' })
  })
})
