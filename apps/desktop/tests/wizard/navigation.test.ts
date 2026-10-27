import {
  advance,
  canJumpTo,
  invalidateFrom,
  isPermissible,
  lastCustomizationStep,
  markPermissible,
  nextStepIndex,
  previousStepIndex,
  selectDevice,
  selectOs,
} from '@/wizard/navigation'
import { jump, retreat } from '@/wizard/operations'
import { initialState, type WizardState } from '@/wizard/state'
import { Step } from '@/wizard/stepIds'

const at = (currentStep: number, extra: Partial<WizardState> = {}): WizardState => ({
  ...initialState,
  currentStep,
  ...extra,
})

describe('permissible steps bitmap', () => {
  it('marks, checks and invalidates bits', () => {
    const bits = markPermissible(markPermissible(1, Step.OS), Step.Hostname)
    expect(isPermissible(bits, Step.OS)).toBe(true)
    expect(isPermissible(bits, Step.Storage)).toBe(false)
    expect(invalidateFrom(bits, Step.Storage)).toBe(0b11)
  })
})

describe('nextStepIndex', () => {
  it('walks the basic order and stops after Done', () => {
    expect(nextStepIndex(at(Step.Device))).toBe(Step.OS)
    expect(nextStepIndex(at(Step.Wifi))).toBe(Step.RemoteAccess)
    expect(nextStepIndex(at(Step.Done))).toBeNull()
  })

  it('skips unavailable optional steps going forward', () => {
    expect(nextStepIndex(at(Step.RemoteAccess))).toBe(Step.Writing)
    expect(nextStepIndex(at(Step.RemoteAccess, { piConnectAvailable: true }))).toBe(Step.PiConnect)
    expect(nextStepIndex(at(Step.RemoteAccess, { secureBootAvailable: true }))).toBe(
      Step.SecureBoot,
    )
    expect(nextStepIndex(at(Step.PiConnect, { ccRpiAvailable: true }))).toBe(Step.Writing)
    expect(
      nextStepIndex(at(Step.PiConnect, { ccRpiAvailable: true, ifAndFeaturesAvailable: true })),
    ).toBe(Step.IfAndFeatures)
  })

  it('skips customisation when unsupported and honours Write Another', () => {
    expect(nextStepIndex(at(Step.Storage, { customizationSupported: false }))).toBe(Step.Writing)
    expect(nextStepIndex(at(Step.Storage, { writeAnotherMode: true }))).toBe(Step.Writing)
  })
})

describe('advance', () => {
  it('applies customisation and snapshots flags on entering Writing', () => {
    const state = at(Step.RemoteAccess, {
      wifiConfigured: true,
      sshEnabled: true,
      customizationSettings: { hostname: 'pi' },
    })
    const { state: next, apply } = advance(state)
    expect(apply).toBe(true)
    expect(next.currentStep).toBe(Step.Writing)
    expect(next.customizationSettings).toEqual({ hostname: 'pi', wifiConfigured: true })
    expect(next.completionSnapshot).toMatchObject({
      customizationSupported: true,
      sshEnabled: true,
    })
    expect(isPermissible(next.permissibleStepsBitmap, Step.Writing)).toBe(true)
  })

  it('does not apply when customisation is unsupported, and resets Write Another', () => {
    const { state: next, apply } = advance(
      at(Step.Storage, { writeAnotherMode: true, customizationSupported: false }),
    )
    expect(apply).toBe(false)
    expect(next.writeAnotherMode).toBe(false)
    expect(next.currentStep).toBe(Step.Writing)
  })

  it('keeps Write Another away from Storage and is a no-op at Done', () => {
    expect(advance(at(Step.Device, { writeAnotherMode: true })).state.writeAnotherMode).toBe(true)
    const done = at(Step.Done)
    expect(advance(done).state).toBe(done)
  })
})

describe('previousStepIndex', () => {
  it('skips unavailable optional steps going back', () => {
    expect(previousStepIndex(at(Step.Writing))).toBe(Step.RemoteAccess)
    expect(previousStepIndex(at(Step.Writing, { piConnectAvailable: true }))).toBe(Step.PiConnect)
    expect(previousStepIndex(at(Step.Writing, { secureBootAvailable: true }))).toBe(Step.SecureBoot)
    expect(
      previousStepIndex(at(Step.Writing, { ccRpiAvailable: true, ifAndFeaturesAvailable: true })),
    ).toBe(Step.IfAndFeatures)
  })

  it('returns to Storage from Writing without customisation, and stops before offline Device', () => {
    expect(previousStepIndex(at(Step.Writing, { customizationSupported: false }))).toBe(
      Step.Storage,
    )
    expect(previousStepIndex(at(Step.OS, { hasNetworkConnectivity: false }))).toBeNull()
    expect(previousStepIndex(at(Step.OS))).toBe(Step.Device)
    expect(previousStepIndex(at(Step.Device))).toBeNull()
  })
})

describe('jumps', () => {
  it('allows 0..12 and blocks Device while offline', () => {
    expect(canJumpTo(at(Step.OS), Step.Language)).toBe(false)
    expect(canJumpTo(at(Step.OS), 13)).toBe(false)
    expect(canJumpTo(at(Step.OS, { hasNetworkConnectivity: false }), Step.Device)).toBe(false)
    expect(jump(at(Step.OS), Step.Done).currentStep).toBe(Step.Done)
    expect(jump(at(Step.OS, { hasNetworkConnectivity: false }), Step.Device).currentStep).toBe(
      Step.OS,
    )
    expect(retreat(at(Step.Device)).currentStep).toBe(Step.Device)
    expect(retreat(at(Step.OS)).currentStep).toBe(Step.Device)
  })

  it('finds the last customisation step', () => {
    expect(lastCustomizationStep(initialState)).toBe(Step.RemoteAccess)
    expect(lastCustomizationStep({ ...initialState, secureBootAvailable: true })).toBe(
      Step.SecureBoot,
    )
    expect(lastCustomizationStep({ ...initialState, piConnectAvailable: true })).toBe(
      Step.PiConnect,
    )
    expect(
      lastCustomizationStep({
        ...initialState,
        ccRpiAvailable: true,
        ifAndFeaturesAvailable: true,
      }),
    ).toBe(Step.IfAndFeatures)
  })
})

describe('selection changes', () => {
  const travelled = at(Step.Hostname, {
    permissibleStepsBitmap: 0b1111111,
    previousDeviceName: 'Pi 5',
    selectedDeviceName: 'Pi 5',
    previousOsName: 'RPi OS',
    selectedOsName: 'RPi OS',
    selectedStorageName: 'SD',
    hostnameConfigured: true,
    piConnectAvailable: true,
    passwordlessSudoAvailable: true,
  })

  it('a new device clears OS, storage, customisation and later steps', () => {
    const next = selectDevice(travelled, 'Pi 4', true)
    expect(next.permissibleStepsBitmap).toBe(1)
    expect(next).toMatchObject({
      selectedOsName: '',
      selectedStorageName: '',
      previousOsName: '',
      hostnameConfigured: false,
    })
    expect(next).toMatchObject({
      piConnectAvailable: false,
      passwordlessSudoAvailable: false,
      customizationSupported: true,
    })
  })

  it('the same or first device keeps everything', () => {
    expect(selectDevice(travelled, 'Pi 5', true).permissibleStepsBitmap).toBe(0b1111111)
    expect(selectDevice(initialState, 'Pi 5', true).previousDeviceName).toBe('Pi 5')
    expect(selectDevice(initialState, 'Pi 5', true).selectedDeviceIsPi).toBe(true)
    expect(selectDevice(initialState, 'Intel NUC', false).selectedDeviceIsPi).toBe(false)
  })

  it('a new OS clears storage and later steps', () => {
    const next = selectOs(travelled, 'Ubuntu')
    expect(next.permissibleStepsBitmap).toBe(0b11)
    expect(next).toMatchObject({
      selectedStorageName: '',
      hostnameConfigured: false,
      selectedOsName: 'Ubuntu',
    })
  })
})
