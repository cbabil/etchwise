import { nextStepIndex } from '@/wizard/navigation'
import { resetToWriteStep, resetWizard, skipCustomisation } from '@/wizard/operations'
import { initialState, type WizardState } from '@/wizard/state'
import { Step } from '@/wizard/stepIds'

const at = (currentStep: number, extra: Partial<WizardState> = {}): WizardState => ({
  ...initialState,
  currentStep,
  ...extra,
})

describe('whole-wizard operations', () => {
  it('skip customisation clears flags and jumps to Writing without applying', () => {
    const next = skipCustomisation(
      at(Step.Wifi, { hostnameConfigured: true, sshEnabled: true, customizationSettings: {} }),
      {
        secureBootEnabled: false,
      },
    )
    expect(next.currentStep).toBe(Step.Writing)
    expect(next).toMatchObject({ hostnameConfigured: false, sshEnabled: false })
    expect(next.customizationSettings).toEqual({})
  })

  it('resetWizard goes back to the first step', () => {
    const online = resetWizard(
      at(Step.Done, {
        selectedDeviceName: 'Pi',
        previousDeviceName: 'Pi',
        permissibleStepsBitmap: 8191,
      }),
    )
    expect(online).toMatchObject({
      currentStep: Step.Device,
      permissibleStepsBitmap: 1,
      selectedDeviceName: '',
    })
    expect(resetWizard(at(Step.Done, { hasNetworkConnectivity: false })).currentStep).toBe(Step.OS)
  })

  it('resetToWriteStep returns to Storage in Write Another mode', () => {
    const next = resetToWriteStep(
      at(Step.Done, {
        selectedStorageName: 'SD',
        piConnectEnabled: true,
        customizationSettings: { piConnectEnabled: true, hostname: 'pi' },
      }),
    )
    expect(next).toMatchObject({
      currentStep: Step.Storage,
      writeAnotherMode: true,
      selectedStorageName: '',
      piConnectEnabled: false,
    })
    expect(next.customizationSettings).toEqual({ hostname: 'pi' })
    expect(nextStepIndex(next)).toBe(Step.Writing)
  })
})
