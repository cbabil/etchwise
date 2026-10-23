/**
 * Whole-wizard operations of upstream `WizardContainer.qml` and the steps'
 * shared "Skip customisation" handler, as pure state transitions.
 */
import { canJumpTo, previousStepIndex, selectDevice, withStep } from './navigation'
import type { WizardFlags, WizardState } from './state'
import { Step } from './stepIds'

/** `jumpToStep(step)`; unchanged when the jump is not allowed. */
export function jump(state: WizardState, step: number): WizardState {
  return canJumpTo(state, step) ? withStep(state, step) : state
}

/** `previousStep()`. */
export function retreat(state: WizardState): WizardState {
  const prev = previousStepIndex(state)
  return prev === null ? state : withStep(state, prev)
}

/** The flags every step's skip handler clears before jumping to Writing. */
export const skipResetFlags = {
  hostnameConfigured: false,
  localeConfigured: false,
  userConfigured: false,
  wifiConfigured: false,
  sshEnabled: false,
} as const

/**
 * "Skip customisation": clears the customisation flags (plus the step's
 * own `extra` ones, e.g. `{ secureBootEnabled: false }`) and jumps to the
 * Writing step without applying customisation.
 */
export function skipCustomisation(
  state: WizardState,
  extra: Partial<WizardFlags> = {},
): WizardState {
  return jump({ ...state, ...skipResetFlags, ...extra }, Step.Writing)
}

/** `resetWizard()`: back to the first step with every selection cleared. */
export function resetWizard(state: WizardState): WizardState {
  const first = state.hasNetworkConnectivity ? Step.Device : Step.OS
  const cleared = selectDevice(state, '')
  return {
    ...cleared,
    currentStep: first,
    permissibleStepsBitmap: 1,
    writeAnotherMode: false,
    selectedDeviceName: '',
    selectedOsName: '',
    selectedStorageName: '',
    previousDeviceName: '',
    previousOsName: '',
    hostnameConfigured: false,
    localeConfigured: false,
    userConfigured: false,
    wifiConfigured: false,
    sshEnabled: false,
    piConnectEnabled: false,
    piConnectAvailable: false,
    ccRpiAvailable: false,
    ifI2cEnabled: false,
    ifSpiEnabled: false,
    if1WireEnabled: false,
    ifSerial: '',
    featUsbGadgetEnabled: false,
    supportsSerialConsoleOnly: false,
    supportsUsbGadget: false,
  }
}

/** `resetToWriteStep()`: "Write another" — pick new storage, then go straight to Writing. */
export function resetToWriteStep(state: WizardState): WizardState {
  const customizationSettings = Object.fromEntries(
    Object.entries(state.customizationSettings).filter(([key]) => key !== 'piConnectEnabled'),
  )
  return withStep(
    {
      ...state,
      selectedStorageName: '',
      piConnectEnabled: false,
      customizationSettings,
      writeAnotherMode: true,
    },
    Step.Storage,
  )
}
