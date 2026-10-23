/**
 * Sidebar mapping of upstream `WizardContainer.qml`: wizard steps to the
 * grouped sidebar rows (Device is hidden while offline), the customisation
 * sub-steps, and which rows are clickable. Upstream quirks are kept: the
 * customisation checks use row index 3 even when offline, and the "Secure
 * Boot" sub-item is never underlined and navigates to Hostname.
 */
import { isPermissible, lastCustomizationStep } from './navigation'
import type { WizardState } from './state'
import { FIRST_CUSTOMIZATION_STEP, Step } from './stepIds'

/** Row index upstream hard-codes as the "Customisation" group. */
export const CUSTOMISATION_ROW = 3

const ONLINE_GROUPS = ['Device', 'OS', 'Storage', 'Customisation', 'Writing', 'Done'] as const
const ONLINE_TARGETS = [
  Step.Device,
  Step.OS,
  Step.Storage,
  FIRST_CUSTOMIZATION_STEP,
  Step.Writing,
  Step.Done,
]

/** Source strings (context WizardContainer) of the sidebar rows. */
export function groupNames(online: boolean): readonly string[] {
  return online ? ONLINE_GROUPS : ONLINE_GROUPS.slice(1)
}

export function stepForRow(row: number, online: boolean): number {
  const targets = online ? ONLINE_TARGETS : ONLINE_TARGETS.slice(1)
  return targets[row] ?? (online ? Step.Device : Step.OS)
}

export function rowForStep(step: number, state: WizardState): number {
  const shift = state.hasNetworkConnectivity ? 0 : -1
  if (step === Step.Device) return state.hasNetworkConnectivity ? 0 : -1
  if (step === Step.OS) return 1 + shift
  if (step === Step.Storage) return 2 + shift
  if (step >= FIRST_CUSTOMIZATION_STEP && step <= lastCustomizationStep(state)) return 3 + shift
  if (step === Step.Writing) return 4 + shift
  if (step === Step.Done) return 5 + shift
  return 0
}

export function rowClickable(row: number, state: WizardState): boolean {
  if (state.isWriting) return false
  if (!state.customizationSupported && row === CUSTOMISATION_ROW) return false
  const target = stepForRow(row, state.hasNetworkConnectivity)
  return isPermissible(state.permissibleStepsBitmap, target) || target < state.currentStep
}

export type RowTone = 'active' | 'inactive' | 'disabled'

export function rowTone(row: number, state: WizardState): RowTone {
  const current = rowForStep(state.currentStep, state)
  if (row > current || (row === CUSTOMISATION_ROW && !state.customizationSupported))
    return 'disabled'
  return row === current ? 'active' : 'inactive'
}

export interface SubStep {
  label: string
  /** Step a click navigates to. */
  target: number
  /** Step that underlines this label, or null (Secure Boot never is). */
  currentFor: number | null
  configured: boolean
}

function optionalSubSteps(s: WizardState): SubStep[] {
  const interfaces =
    s.ifI2cEnabled ||
    s.ifSpiEnabled ||
    s.if1WireEnabled ||
    s.ifSerial !== '' ||
    s.featUsbGadgetEnabled
  const secureBoot: SubStep = {
    label: 'Secure Boot',
    target: FIRST_CUSTOMIZATION_STEP,
    currentFor: null,
    configured: s.secureBootEnabled,
  }
  const connect: SubStep = {
    label: 'Raspberry Pi Connect',
    target: Step.PiConnect,
    currentFor: Step.PiConnect,
    configured: s.piConnectEnabled,
  }
  const features: SubStep = {
    label: 'Interfaces & Features',
    target: Step.IfAndFeatures,
    currentFor: Step.IfAndFeatures,
    configured: interfaces,
  }
  return [
    ...(s.secureBootAvailable ? [secureBoot] : []),
    ...(s.piConnectAvailable ? [connect] : []),
    ...(s.ccRpiAvailable && s.ifAndFeaturesAvailable ? [features] : []),
  ]
}

const sub = (label: string, step: number, configured: boolean): SubStep => ({
  label,
  target: step,
  currentFor: step,
  configured,
})

/** Upstream `getCustomizationSubstepLabels()` with each label's step and flag. */
export function subSteps(s: WizardState): SubStep[] {
  if (!s.customizationSupported) return []
  return [
    sub('Hostname', Step.Hostname, s.hostnameConfigured),
    sub('Localisation', Step.Locale, s.localeConfigured),
    sub('User', Step.User, s.userConfigured),
    sub('Wi\u2011Fi', Step.Wifi, s.wifiConfigured),
    sub('Remote access', Step.RemoteAccess, s.sshEnabled),
    ...optionalSubSteps(s),
  ]
}

export function subStepsVisible(row: number, s: WizardState): boolean {
  return row === CUSTOMISATION_ROW && s.customizationSupported && s.currentStep > Step.OS
}

const CUSTOMISATION_STEPS = [
  Step.Hostname,
  Step.Locale,
  Step.User,
  Step.Wifi,
  Step.RemoteAccess,
  Step.PiConnect,
  Step.IfAndFeatures,
]

export function subStepClickable(s: WizardState): boolean {
  if (!s.customizationSupported || s.isWriting || s.currentStep <= Step.OS) return false
  return (
    s.currentStep >= FIRST_CUSTOMIZATION_STEP ||
    s.hostnameConfigured ||
    s.localeConfigured ||
    s.userConfigured ||
    s.wifiConfigured ||
    s.sshEnabled ||
    s.piConnectEnabled ||
    CUSTOMISATION_STEPS.some((step) => isPermissible(s.permissibleStepsBitmap, step))
  )
}

export function subStepIsCurrent(sub: SubStep, s: WizardState): boolean {
  if (s.currentStep < FIRST_CUSTOMIZATION_STEP || s.currentStep > lastCustomizationStep(s))
    return false
  return sub.currentFor === s.currentStep
}

/** Whether clicking a sub-step navigates (upstream sub-item `onClicked`). */
export function subStepNavigates(sub: SubStep, s: WizardState): boolean {
  return (
    s.currentStep !== sub.target &&
    (isPermissible(s.permissibleStepsBitmap, sub.target) || sub.target < s.currentStep)
  )
}
