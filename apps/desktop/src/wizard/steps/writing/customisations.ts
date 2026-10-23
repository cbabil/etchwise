/**
 * The customisation lines upstream `WritingStep.qml` and `DoneStep.qml`
 * list, in their order, as `CommonStrings` source texts.
 */
import type { CompletionSnapshot } from '../../state'

export type CustomisationFlags = Omit<CompletionSnapshot, 'customizationSupported'>

const serialConfigured = (f: CustomisationFlags): boolean =>
  f.ifSerial !== '' && f.ifSerial !== 'Disabled'

const lines: ReadonlyArray<[(f: CustomisationFlags) => boolean, string]> = [
  [(f) => f.hostnameConfigured, 'Hostname configured'],
  [(f) => f.localeConfigured, 'Localisation configured'],
  [(f) => f.userConfigured, 'User account configured'],
  [(f) => f.wifiConfigured, 'Wi‑Fi configured'],
  [(f) => f.sshEnabled, 'SSH enabled'],
  [(f) => f.piConnectEnabled, 'Raspberry Pi Connect enabled'],
  [(f) => f.featUsbGadgetEnabled, 'USB Gadget mode enabled'],
  [(f) => f.ifI2cEnabled, 'I2C enabled'],
  [(f) => f.ifSpiEnabled, 'SPI enabled'],
  [(f) => f.if1WireEnabled, '1-Wire enabled'],
  [serialConfigured, 'Serial configured'],
]

/** The `CommonStrings` texts of every customisation that is on. */
export const customisationLines = (f: CustomisationFlags): string[] =>
  lines.filter(([on]) => on(f)).map(([, text]) => text)

/**
 * Upstream `anyCustomizationsApplied`. The Writing step ignores the
 * interface flags (`withInterfaces` false); the Done step counts them.
 */
export function anyCustomisations(
  f: CustomisationFlags & { customizationSupported: boolean },
  withInterfaces: boolean,
): boolean {
  if (!f.customizationSupported) return false
  const base =
    f.hostnameConfigured ||
    f.localeConfigured ||
    f.userConfigured ||
    f.wifiConfigured ||
    f.sshEnabled ||
    f.piConnectEnabled ||
    f.featUsbGadgetEnabled
  if (base || !withInterfaces) return base
  return f.ifI2cEnabled || f.ifSpiEnabled || f.if1WireEnabled || serialConfigured(f)
}

type Tr = (source: string, ...args: unknown[]) => string

/** Upstream list `Accessible.name`: "2 customizations: A, B". */
export function customisationsAccessibleName(items: readonly string[], tr: Tr): string {
  const noun = items.length === 1 ? tr('customization') : tr('customizations')
  return `${items.length} ${noun}: ${items.join(', ')}`
}
