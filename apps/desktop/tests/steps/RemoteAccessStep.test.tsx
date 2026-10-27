import { fireEvent, screen, waitFor } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { RemoteAccessStep } from '@/wizard/steps/RemoteAccessStep'

import { answer, nextButton, renderStep } from './customHarness'

vi.mock('@/api/imager', () => ({
  call: vi.fn(async () => null),
  onImagerEvent: vi.fn(async () => () => undefined),
}))

const KEY_A = 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKeyDataAlpha alice@laptop'
const KEY_B = 'ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQBravo'
let saved: Record<string, unknown> = {}

const backend = (embedded = false) =>
  answer({
    getSavedCustomisationSettings: () => ({ ...saved }),
    setSavedCustomisationSettings: (map) => void (saved = map as Record<string, unknown>),
    isEmbeddedMode: () => embedded,
  })

const sshSwitch = () => screen.getByRole('switch', { name: /^Enable SSH/ })
const radio = (name: RegExp) => screen.getByRole('radio', { name })
const button = (name: RegExp) => screen.getByRole('button', { name })
const keyInput = () => screen.getByRole('textbox', { name: 'SSH public key input' })

describe('RemoteAccessStep', () => {
  beforeEach(() => {
    saved = {}
  })

  it('starts disabled with password auth and the help link outside embedded mode', async () => {
    backend()
    await renderStep(RemoteAccessStep)
    expect(sshSwitch()).not.toBeChecked()
    expect(radio(/^Use password authentication/)).toBeChecked()
    expect(radio(/^Use password authentication/)).toBeDisabled()
    expect(await screen.findByRole('link', { name: 'Learn about SSH' })).toBeInTheDocument()
    expect(nextButton()).toBeEnabled()
  })

  it('names a non-Pi device and drops the Raspberry Pi help link', async () => {
    backend()
    await renderStep(RemoteAccessStep, {
      selectedDeviceName: 'Intel NUC',
      selectedDeviceIsPi: false,
    })
    expect(screen.getByRole('switch', { name: /control of your Intel NUC$/ })).toBeInTheDocument()
    expect(screen.queryByRole('link', { name: 'Learn about SSH' })).toBeNull()
  })

  it('hides the help link in embedded mode', async () => {
    backend(true)
    await renderStep(RemoteAccessStep)
    await waitFor(() => expect(screen.queryByRole('link')).toBeNull())
  })

  it('saves password authentication', async () => {
    saved = { sshPublicKey: '' }
    backend()
    const { wizard } = await renderStep(RemoteAccessStep, { currentStep: Step.RemoteAccess })
    fireEvent.click(sshSwitch())
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Writing))
    expect(wizard().sshEnabled).toBe(true)
    expect(wizard().customizationSettings).toMatchObject({
      sshEnabled: true,
      sshPasswordAuth: true,
    })
    expect(saved).toEqual({ sshEnabled: true, sshPasswordAuth: true })
  })

  it('manages keys: add, dedupe, show / hide, remove', async () => {
    backend()
    await renderStep(RemoteAccessStep, { currentStep: Step.RemoteAccess })
    fireEvent.click(sshSwitch())
    fireEvent.click(radio(/^Use public key authentication/))
    expect(screen.getByText('No SSH keys configured')).toBeInTheDocument()
    expect(nextButton()).toBeDisabled()
    fireEvent.click(button(/^Show/))
    expect(button(/^Browse/)).toBeInTheDocument()
    fireEvent.change(keyInput(), { target: { value: `  ${KEY_A}  ` } })
    fireEvent.click(button(/^Add/))
    fireEvent.change(keyInput(), { target: { value: KEY_A } })
    fireEvent.keyDown(keyInput(), { key: 'Enter' })
    expect(screen.getByText('1 SSH key configured')).toBeInTheDocument()
    fireEvent.change(keyInput(), { target: { value: KEY_B } })
    fireEvent.keyDown(keyInput(), { key: 'Enter' })
    expect(screen.getByText('2 SSH keys configured')).toBeInTheDocument()
    expect(
      screen.getByRole('group', { name: 'SSH key 1: ssh-ed25519, alice@laptop' }),
    ).toHaveTextContent('ssh-ed25519 AAAAC3Nz...ataAlpha alice@laptop')
    fireEvent.click(button(/^Remove, Remove SSH key: alice@laptop/))
    expect(screen.getByText('1 SSH key configured')).toBeInTheDocument()
    expect(button(/^Remove, Remove SSH key 1$/)).toBeInTheDocument()
    fireEvent.click(button(/^Hide/))
    expect(screen.queryByRole('textbox')).toBeNull()
    expect(nextButton()).toBeEnabled()
  })

  it('loads saved keys, switches to public key and saves them', async () => {
    saved = { sshEnabled: 'true', sshAuthorizedKeys: `${KEY_A}\r\n\n${KEY_B}\n`, sshPublicKey: 'x' }
    backend()
    const { wizard } = await renderStep(RemoteAccessStep, { currentStep: Step.RemoteAccess })
    expect(sshSwitch()).toBeChecked()
    expect(radio(/^Use public key authentication/)).toBeChecked()
    expect(screen.getByText('2 SSH keys configured')).toBeInTheDocument()
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Writing))
    const expected = {
      sshEnabled: true,
      sshPasswordAuth: false,
      sshAuthorizedKeys: `${KEY_A}\n${KEY_B}`,
    }
    expect(wizard().customizationSettings).toMatchObject(expected)
    expect(wizard().customizationSettings).not.toHaveProperty('sshPublicKey')
    expect(saved).toEqual(expected)
  })

  it('falls back to the legacy sshPublicKey and clears everything when disabled', async () => {
    saved = { sshEnabled: true, sshPublicKey: KEY_B, sshPasswordAuth: false, hostname: 'pi' }
    backend()
    const { wizard } = await renderStep(RemoteAccessStep, { currentStep: Step.RemoteAccess })
    expect(screen.getByText('1 SSH key configured')).toBeInTheDocument()
    fireEvent.click(sshSwitch())
    fireEvent.click(nextButton())
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Writing))
    expect(wizard().sshEnabled).toBe(false)
    expect(wizard().customizationSettings).not.toHaveProperty('sshPublicKey')
    expect(saved).toEqual({ hostname: 'pi' })
  })
})
