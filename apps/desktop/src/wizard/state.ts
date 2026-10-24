/**
 * Wizard state mirroring the properties of upstream `WizardContainer.qml`.
 * Everything is replaced, never mutated.
 */
import { sizes } from '@/theme/tokens'

import { Step } from './stepIds'

export type CustomizationSettings = Readonly<Record<string, unknown>>

/** Flags steps read and write (upstream container properties). */
export interface WizardFlags {
  isWriting: boolean
  supportsSerialConsoleOnly: boolean
  supportsUsbGadget: boolean
  hostnameConfigured: boolean
  localeConfigured: boolean
  userConfigured: boolean
  wifiConfigured: boolean
  sshEnabled: boolean
  secureBootEnabled: boolean
  piConnectEnabled: boolean
  piConnectAvailable: boolean
  secureBootAvailable: boolean
  passwordlessSudoAvailable: boolean
  secureBootKeyConfigured: boolean
  ccRpiAvailable: boolean
  /** The image sets itself up through Ubuntu autoinstall (`init_format`). */
  autoinstall: boolean
  ifAndFeaturesAvailable: boolean
  ifI2cEnabled: boolean
  ifSpiEnabled: boolean
  if1WireEnabled: boolean
  /** "Disabled" | "Default" | "Console & Hardware" | "Console" | "Hardware" | "" */
  ifSerial: string
  featUsbGadgetEnabled: boolean
  disableWarnings: boolean
  customizationSupported: boolean
}

/** Upstream `completionSnapshot`, captured on entering the Writing step. */
export type CompletionSnapshot = Pick<
  WizardFlags,
  | 'customizationSupported'
  | 'hostnameConfigured'
  | 'localeConfigured'
  | 'userConfigured'
  | 'wifiConfigured'
  | 'sshEnabled'
  | 'piConnectEnabled'
  | 'ifI2cEnabled'
  | 'ifSpiEnabled'
  | 'if1WireEnabled'
  | 'ifSerial'
  | 'featUsbGadgetEnabled'
>

export interface WizardState extends WizardFlags {
  currentStep: number
  /** Bit n set = step n may be jumped to from the sidebar. */
  permissibleStepsBitmap: number
  writeAnotherMode: boolean
  hasNetworkConnectivity: boolean
  showLanguageSelection: boolean
  selectedDeviceName: string
  selectedOsName: string
  selectedStorageName: string
  previousDeviceName: string
  previousOsName: string
  customizationSettings: CustomizationSettings
  completionSnapshot: CompletionSnapshot
  sidebarWidth: number
}

export const customizationResetFlags = {
  hostnameConfigured: false,
  localeConfigured: false,
  userConfigured: false,
  wifiConfigured: false,
  sshEnabled: false,
  piConnectEnabled: false,
  ifI2cEnabled: false,
  ifSpiEnabled: false,
  if1WireEnabled: false,
  ifSerial: '',
  featUsbGadgetEnabled: false,
} as const

export const initialFlags: WizardFlags = {
  ...customizationResetFlags,
  isWriting: false,
  supportsSerialConsoleOnly: false,
  supportsUsbGadget: false,
  secureBootEnabled: false,
  piConnectAvailable: false,
  secureBootAvailable: false,
  passwordlessSudoAvailable: false,
  secureBootKeyConfigured: false,
  ccRpiAvailable: false,
  autoinstall: false,
  ifAndFeaturesAvailable: false,
  disableWarnings: false,
  customizationSupported: true,
}

export function snapshotOf(flags: WizardFlags): CompletionSnapshot {
  return {
    customizationSupported: flags.customizationSupported,
    hostnameConfigured: flags.hostnameConfigured,
    localeConfigured: flags.localeConfigured,
    userConfigured: flags.userConfigured,
    wifiConfigured: flags.wifiConfigured,
    sshEnabled: flags.sshEnabled,
    piConnectEnabled: flags.piConnectEnabled,
    ifI2cEnabled: flags.ifI2cEnabled,
    ifSpiEnabled: flags.ifSpiEnabled,
    if1WireEnabled: flags.if1WireEnabled,
    ifSerial: flags.ifSerial,
    featUsbGadgetEnabled: flags.featUsbGadgetEnabled,
  }
}

export const initialState: WizardState = {
  ...initialFlags,
  currentStep: Step.Device,
  permissibleStepsBitmap: 1,
  writeAnotherMode: false,
  hasNetworkConnectivity: true,
  showLanguageSelection: false,
  selectedDeviceName: '',
  selectedOsName: '',
  selectedStorageName: '',
  previousDeviceName: '',
  previousOsName: '',
  customizationSettings: {},
  completionSnapshot: { ...snapshotOf(initialFlags), customizationSupported: false },
  sidebarWidth: sizes.sidebarWidth,
}
