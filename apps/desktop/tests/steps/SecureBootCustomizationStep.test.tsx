import { fireEvent, screen, waitFor } from '@testing-library/react'

import { pickFile } from '@/wizard/dialogs/common/pickFile'
import { Step } from '@/wizard/stepIds'
import {
  elideMiddle,
  SecureBootCustomizationStep,
} from '@/wizard/steps/SecureBootCustomizationStep'

import { callsOf, setAnswers } from '../dialogs/imagerMock'
import { renderInWizard } from '../dialogs/renderWizard'

vi.mock('@/api/imager', () => import('../dialogs/imagerMock'))
vi.mock('@/wizard/dialogs/common/pickFile', () => ({ pickFile: vi.fn() }))

const signing = () => screen.getByRole('switch', { name: /^Enable Secure Boot Signing/ })

function backend(extra: Record<string, (...a: unknown[]) => unknown> = {}) {
  setAnswers({
    getStringSetting: (key) => (key === 'secureboot_rsa_key' ? '/home/pi/.ssh/key.pem' : ''),
    getRsaKeyFingerprint: () => 'ab:cd:ef',
    getSavedCustomisationSettings: () => ({ hostname: 'pi', secureBootEnabled: true }),
    ...extra,
  })
}

const renderStep = () =>
  renderInWizard(<SecureBootCustomizationStep />, {
    step: Step.SecureBoot,
    initial: { secureBootAvailable: true },
  })

describe('SecureBootCustomizationStep', () => {
  it('keeps signing disabled and warns until a key is selected', async () => {
    setAnswers({})
    await renderStep()
    expect(signing()).toBeDisabled()
    expect(screen.getByText(/Please select an RSA private key above/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /^RSA Private Key - Select/ })).toBeInTheDocument()
  })

  it('shows the selected key and the fingerprint once enabled, then saves on Next', async () => {
    backend({ getSavedCustomisationSettings: () => ({ hostname: 'pi' }) })
    const { wizard } = await renderStep()
    expect(await screen.findByText('Selected: /home/pi/.ssh/key.pem')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /^RSA Private Key - Change/ })).toBeInTheDocument()
    await waitFor(() => expect(signing()).toBeEnabled())
    fireEvent.click(signing())
    expect(wizard().secureBootEnabled).toBe(true)
    expect(await screen.findByText('Public Key Fingerprint: ab:cd:ef')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: /^Next/ }))
    expect(wizard().customizationSettings.secureBootEnabled).toBe(true)
    expect(wizard().currentStep).toBe(Step.Writing)
    await waitFor(() =>
      expect(callsOf('setSavedCustomisationSettings')).toEqual([
        [{ hostname: 'pi', secureBootEnabled: true }],
      ]),
    )
  })

  it('prefills from saved settings and removes the setting when switched off', async () => {
    backend({ getRsaKeyFingerprint: () => '' })
    const { wizard } = await renderStep()
    expect(signing()).toBeChecked()
    expect(
      await screen.findByText('Public Key Fingerprint: (unable to compute)'),
    ).toBeInTheDocument()
    await waitFor(() => expect(signing()).toBeEnabled())
    fireEvent.click(signing())
    fireEvent.click(screen.getByRole('button', { name: /^Next/ }))
    expect(wizard().customizationSettings.secureBootEnabled).toBeUndefined()
    expect(wizard().secureBootEnabled).toBe(false)
    await waitFor(() =>
      expect(callsOf('setSavedCustomisationSettings')).toEqual([[{ hostname: 'pi' }]]),
    )
  })

  it('stores a newly picked key and marks the key configured', async () => {
    setAnswers({})
    vi.mocked(pickFile).mockResolvedValueOnce(null).mockResolvedValueOnce('/keys/new.pem')
    const { wizard } = await renderStep()
    const select = screen.getByRole('button', { name: /^RSA Private Key - Select/ })
    fireEvent.click(select)
    await waitFor(() => expect(pickFile).toHaveBeenCalledTimes(1))
    expect(callsOf('setSetting')).toEqual([])
    fireEvent.click(select)
    expect(await screen.findByText('Selected: /keys/new.pem')).toBeInTheDocument()
    expect(callsOf('setSetting')).toEqual([['secureboot_rsa_key', '/keys/new.pem']])
    expect(wizard().secureBootKeyConfigured).toBe(true)
    expect(vi.mocked(pickFile).mock.calls[0]?.[0]).toMatchObject({ homeSubdir: '.ssh' })
  })

  it('shows the OTP provisioning box for rpiboot devices', async () => {
    backend({ isRpibootDevice: () => true })
    await renderStep()
    const otp = await screen.findByRole('switch', { name: /^Provision Secure Boot OTP/ })
    await waitFor(() => expect(otp).toBeEnabled())
    fireEvent.click(otp)
    expect(screen.getByText(/WARNING: After OTP provisioning/)).toBeInTheDocument()
  })

  it('skips customisation and clears the Secure Boot flag', async () => {
    setAnswers({})
    const { wizard } = await renderStep()
    fireEvent.click(screen.getByRole('button', { name: /^Skip customisation/ }))
    expect(wizard().currentStep).toBe(Step.Writing)
    expect(wizard().secureBootEnabled).toBe(false)
  })

  it('elides long paths in the middle', () => {
    expect(elideMiddle('short')).toBe('short')
    const long = `/${'a'.repeat(40)}/${'b'.repeat(40)}.pem`
    const elided = elideMiddle(long, 21)
    expect(elided).toHaveLength(21)
    expect(elided.startsWith('/aaaa')).toBe(true)
    expect(elided.endsWith('b.pem')).toBe(true)
  })
})
