/** Wizard step indices, exactly as upstream `WizardContainer.qml`. */
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
  Writing: 11,
  Done: 12,
} as const

export type StepId = (typeof Step)[keyof typeof Step]

export const TOTAL_STEPS = 13
export const FIRST_CUSTOMIZATION_STEP = Step.Hostname
