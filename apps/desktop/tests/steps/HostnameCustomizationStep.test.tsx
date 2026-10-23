import { fireEvent, screen } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { HostnameCustomizationStep } from '@/wizard/steps/HostnameCustomizationStep'

import { answer, callsTo, nextButton, renderStep } from './customHarness'

vi.mock('@/api/imager', () => ({
  call: vi.fn(async () => null),
  onImagerEvent: vi.fn(async () => () => undefined),
}))

const field = () => screen.getByRole('textbox', { name: 'Enter your hostname' })

describe('HostnameCustomizationStep', () => {
  it('prefills a saved hostname and marks it configured', async () => {
    answer({ getSavedCustomisationSettings: () => ({ hostname: 'saved-pi' }) })
    const { wizard } = await renderStep(HostnameCustomizationStep)
    expect(field()).toHaveValue('saved-pi')
    expect(wizard().hostnameConfigured).toBe(true)
    expect(
      screen.getByText(/A hostname is a unique name that identifies your Raspberry Pi/),
    ).toBeInTheDocument()
  })

  it('restricts input to the hostname pattern', async () => {
    answer({})
    await renderStep(HostnameCustomizationStep)
    fireEvent.change(field(), { target: { value: 'my-pi' } })
    expect(field()).toHaveValue('my-pi')
    fireEvent.change(field(), { target: { value: '-bad' } })
    fireEvent.change(field(), { target: { value: 'my pi' } })
    fireEvent.change(field(), { target: { value: 'a'.repeat(64) } })
    expect(field()).toHaveValue('my-pi')
    fireEvent.change(field(), { target: { value: '' } })
    expect(field()).toHaveValue('')
  })

  it('stores the trimmed hostname on Next and persists it', async () => {
    answer({})
    const { wizard } = await renderStep(HostnameCustomizationStep, { currentStep: Step.Hostname })
    fireEvent.change(field(), { target: { value: 'pi4' } })
    fireEvent.click(nextButton())
    expect(wizard().customizationSettings.hostname).toBe('pi4')
    expect(wizard().hostnameConfigured).toBe(true)
    expect(wizard().currentStep).toBe(Step.Locale)
    expect(callsTo('setPersistedCustomisationSetting')).toEqual([['hostname', 'pi4']])
  })

  it('removes the hostname when left empty', async () => {
    answer({ getSavedCustomisationSettings: () => ({ hostname: 'old' }) })
    const { wizard } = await renderStep(HostnameCustomizationStep, { currentStep: Step.Hostname })
    fireEvent.change(field(), { target: { value: '' } })
    fireEvent.click(nextButton())
    expect(wizard().customizationSettings).not.toHaveProperty('hostname')
    expect(wizard().hostnameConfigured).toBe(false)
    expect(callsTo('removePersistedCustomisationSetting')).toEqual([['hostname']])
  })

  it('skips customisation and goes back', async () => {
    answer({})
    const { wizard } = await renderStep(HostnameCustomizationStep, {
      currentStep: Step.Hostname,
      sshEnabled: true,
    })
    fireEvent.click(screen.getByRole('button', { name: /^Back/ }))
    expect(wizard().currentStep).toBe(Step.Storage)
    fireEvent.click(screen.getByRole('button', { name: /^Skip customisation/ }))
    expect(wizard()).toMatchObject({ currentStep: Step.Writing, sshEnabled: false })
  })
})
