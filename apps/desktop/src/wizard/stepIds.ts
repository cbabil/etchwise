/** Wizard step indices: upstream `WizardContainer.qml`, plus Etchwise's DiskLayout. */
export const Step = {
  Language: -1,
  Device: 0,
  OS: 1,
  Storage: 2,
  Hostname: 3,
  Locale: 4,
  User: 5,
  Wifi: 6,
  RemoteAccess: 7,
  SecureBoot: 8,
  PiConnect: 9,
  IfAndFeatures: 10,
  /** Etchwise: Kura's disk layout (only for Kura images). */
  DiskLayout: 11,
  Writing: 12,
  Done: 13,
} as const

export type StepId = (typeof Step)[keyof typeof Step]

export const TOTAL_STEPS = 14
export const FIRST_CUSTOMIZATION_STEP = Step.Hostname
