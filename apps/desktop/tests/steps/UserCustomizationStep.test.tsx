import { fireEvent, screen, waitFor } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { UserCustomizationStep } from '@/wizard/steps/UserCustomizationStep'

import { answer, callsTo, nextButton, renderStep } from './customHarness'

vi.mock('@/api/imager', () => ({
  call: vi.fn(async () => null),
  onImagerEvent: vi.fn(async () => () => undefined),
}))

const username = () => screen.getByLabelText('Username:')
const password = () => screen.getByLabelText('Password:')
const confirm = () => screen.getByLabelText('Confirm password:')
const type = (el: HTMLElement, value: string) => fireEvent.change(el, { target: { value } })

describe('UserCustomizationStep', () => {
  it('restricts the username and enables Next per upstream rules', async () => {
    answer({})
    await renderStep(UserCustomizationStep)
    expect(nextButton()).toBeEnabled()
    type(username(), 'Pi')
    expect(username()).toHaveValue('')
    type(username(), 'pi_user-1')
    expect(username()).toHaveValue('pi_user-1')
    expect(nextButton()).toBeDisabled()
    type(password(), 'secret')
    expect(nextButton()).toBeDisabled()
    type(confirm(), 'secreT')
    expect(nextButton()).toBeDisabled()
    type(confirm(), 'secret')
    expect(nextButton()).toBeEnabled()
    expect(screen.queryByRole('checkbox', { name: /passwordless sudo/ })).toBeNull()
  })

  it('hashes a new password on Next and never stores plaintext', async () => {
    answer({ crypt: (pw) => `$y$hash-of-${String(pw).length}` })
    const { wizard } = await renderStep(UserCustomizationStep, { currentStep: Step.User })
    type(username(), 'pi')
    type(password(), 'secret')
    type(confirm(), 'secret')
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Wifi))
    expect(wizard().customizationSettings).toMatchObject({
      sshUserName: 'pi',
      sshUserPassword: '$y$hash-of-6',
    })
    expect(wizard().userConfigured).toBe(true)
    expect(callsTo('setPersistedCustomisationSetting')).toEqual([
      ['sshUserName', 'pi'],
      ['sshUserPassword', '$y$hash-of-6'],
    ])
    expect(JSON.stringify(wizard().customizationSettings)).not.toContain('secret')
  })

  it('keeps a saved hash when the password fields stay blank', async () => {
    answer({
      getSavedCustomisationSettings: () => ({ sshUserName: 'old', sshUserPassword: '$y$saved' }),
    })
    const { wizard } = await renderStep(UserCustomizationStep, { currentStep: Step.User })
    expect(username()).toHaveValue('old')
    expect(password()).toHaveAttribute('placeholder', 'Saved (hidden) — leave blank to keep')
    expect(confirm()).toHaveAttribute('placeholder', 'Re-enter to change password')
    type(username(), 'new')
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Wifi))
    expect(wizard().customizationSettings).toMatchObject({
      sshUserName: 'new',
      sshUserPassword: '$y$saved',
    })
    expect(callsTo('crypt')).toEqual([])
    expect(callsTo('setPersistedCustomisationSetting')).toEqual([['sshUserName', 'new']])
  })

  it('removes the account when every field is cleared', async () => {
    answer({
      getSavedCustomisationSettings: () => ({ sshUserName: 'old', sshUserPassword: '$y$saved' }),
    })
    const { wizard } = await renderStep(UserCustomizationStep, { currentStep: Step.User })
    type(username(), '')
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Wifi))
    expect(wizard().customizationSettings).not.toHaveProperty('sshUserName')
    expect(wizard().customizationSettings).not.toHaveProperty('sshUserPassword')
    expect(wizard().userConfigured).toBe(false)
    expect(callsTo('removePersistedCustomisationSetting')).toEqual([
      ['sshUserName'],
      ['sshUserPassword'],
    ])
  })

  it('leaves a password without username unconfigured', async () => {
    answer({ crypt: () => 'x' })
    const { wizard } = await renderStep(UserCustomizationStep, {
      currentStep: Step.User,
      userConfigured: true,
    })
    type(password(), 'pw')
    type(confirm(), 'pw')
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Wifi))
    expect(wizard().userConfigured).toBe(false)
  })

  it('confirms passwordless sudo through the warning dialog', async () => {
    answer({ crypt: () => '$y$h' })
    const { wizard } = await renderStep(UserCustomizationStep, {
      currentStep: Step.User,
      passwordlessSudoAvailable: true,
    })
    const sudo = screen.getByRole('checkbox', { name: 'Enable passwordless sudo' })
    fireEvent.click(sudo)
    expect(screen.getByRole('dialog', { name: 'Passwordless Sudo' })).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: /^CANCEL/ }))
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
    expect(sudo).not.toBeChecked()
    fireEvent.click(sudo)
    fireEvent.click(screen.getByRole('button', { name: /^ENABLE/ }))
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
    expect(sudo).toBeChecked()
    expect(screen.getByRole('note', { name: /^Passwordless sudo information/ })).toBeVisible()
    type(username(), 'pi')
    type(password(), 'pw')
    type(confirm(), 'pw')
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().customizationSettings.passwordlessSudo).toBe(true))
    expect(callsTo('setPersistedCustomisationSetting').map(([key]) => key)).not.toContain(
      'passwordlessSudo',
    )
  })
})
