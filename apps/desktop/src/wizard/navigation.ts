/**
 * Pure navigation rules of upstream `WizardContainer.qml`: next / previous
 * with optional-step skipping, the permissible-steps bitmap, jumps, and
 * the invalidation that follows a device or OS change.
 */
import { customizationResetFlags, snapshotOf, type WizardFlags, type WizardState } from './state'
import { FIRST_CUSTOMIZATION_STEP, Step, TOTAL_STEPS } from './stepIds'

export const markPermissible = (bitmap: number, step: number): number => bitmap | (1 << step)
export const isPermissible = (bitmap: number, step: number): boolean => (bitmap & (1 << step)) !== 0
export const invalidateFrom = (bitmap: number, from: number): number => bitmap & ((1 << from) - 1)

type Availability = Pick<
  WizardFlags,
  'ccRpiAvailable' | 'ifAndFeaturesAvailable' | 'piConnectAvailable' | 'secureBootAvailable'
>

export function lastCustomizationStep(f: Availability): number {
  if (f.ccRpiAvailable && f.ifAndFeaturesAvailable) return Step.IfAndFeatures
  if (f.piConnectAvailable) return Step.PiConnect
  if (f.secureBootAvailable) return Step.SecureBoot
  return Step.RemoteAccess
}

/** Sets the current step and marks it permissible (upstream `onCurrentStepChanged`). */
export function withStep(state: WizardState, step: number): WizardState {
  return {
    ...state,
    currentStep: step,
    permissibleStepsBitmap: markPermissible(state.permissibleStepsBitmap, step),
  }
}

/** Target of `nextStep()` from `state`, or null at the last step. */
export function nextStepIndex(state: WizardState): number | null {
  if (state.currentStep >= TOTAL_STEPS - 1) return null
  let next = state.currentStep + 1
  if (state.writeAnotherMode && state.currentStep === Step.Storage) next = Step.Writing
  else if (!state.customizationSupported && next === FIRST_CUSTOMIZATION_STEP) next = Step.Writing
  if (!state.secureBootAvailable && next === Step.SecureBoot) next++
  if (!state.piConnectAvailable && next === Step.PiConnect) next++
  if ((!state.ccRpiAvailable || !state.ifAndFeaturesAvailable) && next === Step.IfAndFeatures)
    next++
  return next
}

/** State after `nextStep()`; `apply` tells the caller to push customisation to the backend. */
export function advance(state: WizardState): { state: WizardState; apply: boolean } {
  const next = nextStepIndex(state)
  if (next === null) return { state, apply: false }
  const writeAnotherMode =
    state.writeAnotherMode && state.currentStep === Step.Storage ? false : state.writeAnotherMode
  let result: WizardState = { ...state, writeAnotherMode }
  let apply = false
  if (next === Step.Writing) {
    apply = state.customizationSupported
    const customizationSettings = apply
      ? { ...state.customizationSettings, wifiConfigured: state.wifiConfigured }
      : state.customizationSettings
    result = { ...result, customizationSettings, completionSnapshot: snapshotOf(state) }
  }
  return { state: withStep(result, next), apply }
}

/** Target of `previousStep()`, or null when it stays put. */
export function previousStepIndex(state: WizardState): number | null {
  if (state.currentStep <= 0) return null
  let prev = state.currentStep - 1
  if (state.currentStep === Step.Writing && !state.customizationSupported) return Step.Storage
  if (prev === Step.IfAndFeatures && (!state.ccRpiAvailable || !state.ifAndFeaturesAvailable))
    prev--
  if (prev === Step.PiConnect && !state.piConnectAvailable) prev--
  if (prev === Step.SecureBoot && !state.secureBootAvailable) prev--
  if (prev === Step.Device && !state.hasNetworkConnectivity) return null
  return prev
}

/** Upstream `jumpToStep` guard: 0…12 and never Device while offline. */
export function canJumpTo(state: WizardState, step: number): boolean {
  if (step < 0 || step >= TOTAL_STEPS) return false
  return !(step === Step.Device && !state.hasNetworkConnectivity)
}

const capabilityReset = {
  piConnectAvailable: false,
  secureBootAvailable: false,
  ccRpiAvailable: false,
} as const

/** Upstream `onSelectedDeviceNameChanged` + `invalidateDeviceDependentSteps`. */
export function selectDevice(state: WizardState, name: string, isPi: boolean): WizardState {
  const changed = state.previousDeviceName !== '' && state.previousDeviceName !== name
  const base = {
    ...state,
    selectedDeviceName: name,
    selectedDeviceIsPi: isPi,
    previousDeviceName: name,
  }
  if (!changed) return base
  // Clearing the OS name re-triggers upstream's OS-change handler.
  return selectOs(
    {
      ...base,
      ...customizationResetFlags,
      ...capabilityReset,
      secureBootEnabled: false,
      permissibleStepsBitmap: invalidateFrom(state.permissibleStepsBitmap, Step.OS),
      selectedStorageName: '',
      customizationSupported: true,
    },
    '',
  )
}

/** Upstream `onSelectedOsNameChanged` + `invalidateOSDependentSteps`. */
export function selectOs(state: WizardState, name: string): WizardState {
  const changed = state.previousOsName !== '' && state.previousOsName !== name
  const base = { ...state, selectedOsName: name, previousOsName: name }
  if (!changed) return base
  return {
    ...base,
    ...customizationResetFlags,
    ...capabilityReset,
    passwordlessSudoAvailable: false,
    permissibleStepsBitmap: invalidateFrom(state.permissibleStepsBitmap, Step.Storage),
    selectedStorageName: '',
  }
}
