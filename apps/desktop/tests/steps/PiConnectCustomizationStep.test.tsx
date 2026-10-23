import { act, fireEvent, screen, waitFor } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { connectReducer, initialConnectState } from '@/wizard/steps/custom2/piConnectState'
import { CONNECT_AUTH_URL } from '@/wizard/steps/custom2/usePiConnect'
import { PiConnectCustomizationStep } from '@/wizard/steps/PiConnectCustomizationStep'

import { callsOf, emit, setAnswers } from '../dialogs/imagerMock'
import { renderInWizard } from '../dialogs/renderWizard'

vi.mock('@/api/imager', () => import('../dialogs/imagerMock'))

const TOKEN = 'rpuak_abcdefghijkmnpqrstuvwxyz123'
const pill = () => screen.getByRole('switch', { name: /^Enable Raspberry Pi Connect/ })
const field = () => screen.getByRole('textbox', { name: 'Authentication token:' })
const nextButton = () => screen.getByRole('button', { name: /^Next/ })

const renderStep = () =>
  renderInWizard(<PiConnectCustomizationStep />, {
    step: Step.PiConnect,
    initial: { piConnectAvailable: true },
  })

afterEach(() => {
  vi.useRealTimers()
})

describe('PiConnectCustomizationStep', () => {
  it('counts down 25 s after opening Connect, then unlocks the token field', async () => {
    setAnswers({})
    await renderStep()
    expect(screen.queryByRole('button', { name: /^Open Raspberry Pi Connect/ })).toBeNull()
    fireEvent.click(pill())
    expect(field()).toBeDisabled()
    expect(field()).toHaveAttribute('placeholder', 'Waiting for token (25s)')
    expect(nextButton()).toBeDisabled()
    vi.useFakeTimers()
    fireEvent.click(screen.getByRole('button', { name: /^Open Raspberry Pi Connect/ }))
    expect(callsOf('openUrl')).toEqual([[CONNECT_AUTH_URL]])
    act(() => vi.advanceTimersByTime(1000))
    expect(field()).toHaveAttribute('placeholder', 'Waiting for token (24s)')
    fireEvent.click(screen.getByRole('button', { name: /^Open Raspberry Pi Connect/ }))
    act(() => vi.advanceTimersByTime(24000))
    expect(field()).toHaveAttribute('placeholder', 'Paste token here')
    expect(field()).toBeEnabled()
  })

  async function unlockField() {
    fireEvent.click(pill())
    vi.useFakeTimers()
    fireEvent.click(screen.getByRole('button', { name: /^Open Raspberry Pi Connect/ }))
    act(() => vi.advanceTimersByTime(25000))
    vi.useRealTimers()
  }

  it('rejects a hand-typed invalid token and clears it on OK', async () => {
    setAnswers({ verifyAuthKey: () => false })
    await renderStep()
    await unlockField()
    fireEvent.change(field(), { target: { value: '  bogus  ' } })
    expect(nextButton()).toBeEnabled()
    fireEvent.click(nextButton())
    expect(await screen.findByRole('dialog', { name: 'Invalid Token' })).toBeInTheDocument()
    expect(callsOf('verifyAuthKey')).toEqual([['bogus', true]])
    fireEvent.click(screen.getByRole('button', { name: /^OK/ }))
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
    expect(field()).toHaveValue('')
    expect(nextButton()).toBeDisabled()
  })

  it('closes the invalid-token dialog on Escape without clearing the token', async () => {
    setAnswers({ verifyAuthKey: () => false })
    await renderStep()
    await unlockField()
    fireEvent.change(field(), { target: { value: 'bogus' } })
    fireEvent.click(nextButton())
    const dialog = await screen.findByRole('dialog', { name: 'Invalid Token' })
    fireEvent.keyDown(dialog, { key: 'Escape' })
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
    expect(field()).toHaveValue('bogus')
  })

  it('stores a valid hand-typed token and enables Connect', async () => {
    setAnswers({ verifyAuthKey: () => true })
    const { wizard } = await renderStep()
    await unlockField()
    fireEvent.change(field(), { target: { value: TOKEN } })
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Writing))
    expect(callsOf('overwriteConnectToken')).toEqual([[TOKEN]])
    expect(wizard().customizationSettings.piConnectEnabled).toBe(true)
    expect(wizard().piConnectEnabled).toBe(true)
  })

  it('uses a runtime token read-only and skips validation', async () => {
    setAnswers({ getRuntimeConnectToken: () => TOKEN })
    const { wizard } = await renderStep()
    await waitFor(() => expect(pill()).toBeChecked())
    expect(field()).toHaveValue(TOKEN)
    expect(field()).toBeDisabled()
    expect(screen.queryByRole('button', { name: /^Open Raspberry Pi Connect/ })).toBeNull()
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Writing))
    expect(callsOf('verifyAuthKey')).toEqual([])
  })

  it('fills the token from the browser and resets when it is cleared', async () => {
    setAnswers({})
    const { wizard } = await renderStep()
    fireEvent.click(pill())
    emit('connectTokenReceived', TOKEN)
    expect(field()).toHaveValue(TOKEN)
    expect(field()).toHaveAttribute('placeholder', 'Token received from browser')
    emit('connectTokenCleared')
    expect(pill()).not.toBeChecked()
    expect(wizard().piConnectEnabled).toBe(false)
  })

  it('removes the setting when Connect stays off, and skips', async () => {
    setAnswers({})
    const { wizard } = await renderStep()
    fireEvent.click(pill())
    fireEvent.click(pill())
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Writing))
    expect(wizard().customizationSettings.piConnectEnabled).toBeUndefined()
    act(() => wizard().jumpTo(Step.PiConnect))
    fireEvent.click(await screen.findByRole('button', { name: /^Skip customisation/ }))
    expect(wizard().currentStep).toBe(Step.Writing)
  })
})

describe('connectReducer', () => {
  it('ignores ticks while stopped, empty browser tokens and restarts while running', () => {
    expect(connectReducer(initialConnectState, { type: 'tick' })).toBe(initialConnectState)
    expect(connectReducer(initialConnectState, { type: 'browserToken', token: '' })).toBe(
      initialConnectState,
    )
    const running = connectReducer(initialConnectState, { type: 'start' })
    expect(connectReducer(running, { type: 'start' })).toBe(running)
    const typed = connectReducer(running, { type: 'typed', text: ' x ' })
    expect(typed).toMatchObject({ token: 'x', running: false, text: ' x ' })
    expect(connectReducer(typed, { type: 'typed', text: '  ' })).toMatchObject({
      token: '',
      received: false,
    })
  })
})
