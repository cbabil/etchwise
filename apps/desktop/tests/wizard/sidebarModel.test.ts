import {
  groupNames,
  rowClickable,
  rowForStep,
  rowTone,
  stepForRow,
  subStepClickable,
  subStepIsCurrent,
  subStepNavigates,
  subSteps,
  subStepsVisible,
} from '@/wizard/sidebarModel'
import { initialState, type WizardState } from '@/wizard/state'
import { Step } from '@/wizard/stepIds'

const at = (currentStep: number, extra: Partial<WizardState> = {}): WizardState => ({
  ...initialState,
  currentStep,
  ...extra,
})

describe('sidebar rows', () => {
  it('hides Device while offline', () => {
    expect(groupNames(true)).toEqual([
      'Device',
      'OS',
      'Storage',
      'Customisation',
      'Writing',
      'Done',
    ])
    expect(groupNames(false)[0]).toBe('OS')
    expect(stepForRow(3, true)).toBe(Step.Hostname)
    expect(stepForRow(0, false)).toBe(Step.OS)
    expect(stepForRow(9, true)).toBe(Step.Device)
    expect(stepForRow(9, false)).toBe(Step.OS)
  })

  it('maps steps to rows', () => {
    const online = at(Step.Device)
    const offline = at(Step.OS, { hasNetworkConnectivity: false })
    expect(
      [Step.Device, Step.OS, Step.Storage, Step.Wifi, Step.Writing, Step.Done].map((s) =>
        rowForStep(s, online),
      ),
    ).toEqual([0, 1, 2, 3, 4, 5])
    expect([Step.Device, Step.OS, Step.Writing].map((s) => rowForStep(s, offline))).toEqual([
      -1, 0, 3,
    ])
    expect(rowForStep(Step.SecureBoot, online)).toBe(0)
    expect(rowForStep(Step.Language, online)).toBe(0)
  })

  it('tones rows active, inactive and disabled', () => {
    const state = at(Step.Storage)
    expect([0, 1, 2, 3].map((row) => rowTone(row, state))).toEqual([
      'inactive',
      'inactive',
      'active',
      'disabled',
    ])
    expect(rowTone(3, at(Step.Writing, { customizationSupported: false }))).toBe('disabled')
  })

  it('allows clicks on visited or earlier rows only', () => {
    const state = at(Step.Storage, { permissibleStepsBitmap: 0b111 })
    expect(rowClickable(1, state)).toBe(true)
    expect(rowClickable(4, state)).toBe(false)
    expect(rowClickable(1, { ...state, isWriting: true })).toBe(false)
    expect(rowClickable(3, at(Step.Writing, { customizationSupported: false }))).toBe(false)
  })
})

describe('customisation sub-steps', () => {
  it('lists optional steps only when available', () => {
    expect(subSteps(initialState).map((s) => s.label)).toEqual([
      'Hostname',
      'Localisation',
      'User',
      'Wi‑Fi',
      'Remote access',
    ])
    const all = subSteps({
      ...initialState,
      secureBootAvailable: true,
      piConnectAvailable: true,
      ccRpiAvailable: true,
      ifAndFeaturesAvailable: true,
    })
    expect(all.map((s) => s.label).slice(5)).toEqual([
      'Secure Boot',
      'Raspberry Pi Connect',
      'Interfaces & Features',
    ])
    expect(subSteps({ ...initialState, customizationSupported: false })).toEqual([])
  })

  it('marks configured sub-steps', () => {
    const state = {
      ...initialState,
      hostnameConfigured: true,
      ccRpiAvailable: true,
      ifAndFeaturesAvailable: true,
      ifSerial: 'Console',
    }
    const list = subSteps(state)
    expect(list[0]?.configured).toBe(true)
    expect(list[1]?.configured).toBe(false)
    expect(list.at(-1)?.configured).toBe(true)
  })

  it('shows under Customisation after the OS step', () => {
    expect(subStepsVisible(3, at(Step.Storage))).toBe(true)
    expect(subStepsVisible(3, at(Step.OS))).toBe(false)
    expect(subStepsVisible(2, at(Step.Storage))).toBe(false)
  })

  it('underlines the current one (never Secure Boot, as upstream)', () => {
    const state = at(Step.Wifi, { secureBootAvailable: true })
    const list = subSteps(state)
    expect(list.filter((s) => subStepIsCurrent(s, state)).map((s) => s.label)).toEqual(['Wi‑Fi'])
    const secure = at(Step.SecureBoot, { secureBootAvailable: true })
    expect(subSteps(secure).some((s) => subStepIsCurrent(s, secure))).toBe(false)
    expect(subStepIsCurrent(list[0]!, at(Step.Storage))).toBe(false)
  })

  it('is clickable once customisation was reached or configured', () => {
    expect(subStepClickable(at(Step.Storage))).toBe(false)
    expect(subStepClickable(at(Step.Hostname))).toBe(true)
    expect(subStepClickable(at(Step.Storage, { userConfigured: true }))).toBe(true)
    expect(subStepClickable(at(Step.Writing, { permissibleStepsBitmap: 1 << Step.Wifi }))).toBe(
      true,
    )
    expect(subStepClickable(at(Step.Hostname, { isWriting: true }))).toBe(false)
  })

  it('navigates to visited or earlier sub-steps, but not the current one', () => {
    const state = at(Step.User, { permissibleStepsBitmap: (1 << Step.Hostname) | (1 << Step.User) })
    const [hostname, locale, user, wifi] = subSteps(state)
    expect(subStepNavigates(hostname!, state)).toBe(true)
    expect(subStepNavigates(locale!, state)).toBe(true)
    expect(subStepNavigates(user!, state)).toBe(false)
    expect(subStepNavigates(wifi!, state)).toBe(false)
  })
})
