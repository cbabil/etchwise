/** Capability checks and value mapping of upstream `IfAndFeaturesCustomizationStep.qml`. */
import { call } from '@/api/imager'
import type { CustomizationSettings } from '@/wizard/state'

import { isTrue } from './layout'

export interface IfCaps {
  i2c: boolean
  spi: boolean
  onewire: boolean
  serial: boolean
  serialConsoleOnly: boolean
  usbOtg: boolean
}

export interface IfValues {
  i2c: boolean
  spi: boolean
  onewire: boolean
  /** Index into `serialOptions(caps)`. */
  serialIndex: number
  usbGadget: boolean
}

/** Upstream `updateCaps()`. */
export async function loadCaps(): Promise<IfCaps> {
  const sw = (cap: string) => call<boolean | null>('checkHWAndSWCapability', cap)
  const [i2c, spi, onewire, serial, serialConsoleOnly, usbOtg] = await Promise.all([
    sw('i2c'),
    sw('spi'),
    sw('onewire'),
    sw('serial'),
    call<boolean | null>('checkHWCapability', 'serial_on_console_only'),
    sw('usb_otg'),
  ])
  return {
    i2c: Boolean(i2c),
    spi: Boolean(spi),
    onewire: Boolean(onewire),
    serial: Boolean(serial),
    serialConsoleOnly: Boolean(serialConsoleOnly),
    usbOtg: Boolean(usbOtg),
  }
}

export const hasAnyCap = (c: IfCaps): boolean => c.i2c || c.spi || c.onewire || c.serial || c.usbOtg

/** The Serial combo's values; "Console" only on console-only capable devices. */
export function serialOptions(caps: IfCaps): string[] {
  const base = ['Disabled', 'Default', 'Console & Hardware', 'Hardware']
  return caps.serialConsoleOnly ? [...base, 'Console'] : base
}

/** Prefill from the conserved customisation settings and wizard flags. */
export function initialValues(
  caps: IfCaps,
  settings: CustomizationSettings,
  flags: {
    ifI2cEnabled: boolean
    ifSpiEnabled: boolean
    if1WireEnabled: boolean
    ifSerial: string
    featUsbGadgetEnabled: boolean
  },
): IfValues {
  const serial = String(settings.enableSerial || flags.ifSerial || '')
  const index = serialOptions(caps).indexOf(serial)
  return {
    i2c: caps.i2c && (isTrue(settings.enableI2C) || flags.ifI2cEnabled),
    spi: caps.spi && (isTrue(settings.enableSPI) || flags.ifSpiEnabled),
    onewire: caps.onewire && (isTrue(settings.enable1Wire) || flags.if1WireEnabled),
    serialIndex: caps.serial && index >= 0 ? index : 0,
    usbGadget: caps.usbOtg && (isTrue(settings.enableUsbGadget) || flags.featUsbGadgetEnabled),
  }
}

/** Values Next writes: customisation settings (for the generator) and wizard flags. */
export function resultOf(caps: IfCaps, values: IfValues) {
  const selected = serialOptions(caps)[values.serialIndex] ?? 'Disabled'
  const consoleFallback = !caps.serialConsoleOnly && selected === 'Console' ? 'Default' : selected
  const settings = {
    enableI2C: caps.i2c && values.i2c,
    enableSPI: caps.spi && values.spi,
    enable1Wire: caps.onewire && values.onewire,
    enableSerial: caps.serial ? consoleFallback : 'Disabled',
    enableUsbGadget: caps.usbOtg && values.usbGadget,
  }
  const flags = {
    ifI2cEnabled: settings.enableI2C,
    ifSpiEnabled: settings.enableSPI,
    if1WireEnabled: settings.enable1Wire,
    ifSerial: caps.serial ? selected : 'Disabled',
    featUsbGadgetEnabled: settings.enableUsbGadget,
  }
  return { settings, flags }
}

/** Flags cleared when the step is skipped for lack of capabilities. */
export const noIfFeatures = {
  ifI2cEnabled: false,
  ifSpiEnabled: false,
  if1WireEnabled: false,
  featUsbGadgetEnabled: false,
} as const
