/**
 * The backend side of upstream `OSSelectionStep.qml::selectOSitem` for a
 * concrete image or Erase: sets the source, then derives the wizard's
 * capability flags and scrubs settings the new image cannot use.
 */
import { call } from '@/api/imager'

import type { WizardFlags } from '../../state'
import type { WizardApi } from '../../WizardContext'
import type { OsEntry } from './types'

export type SelectionWizard = Pick<
  WizardApi,
  'setSelectedOsName' | 'update' | 'removeCustomizationSetting'
>

const IF_CAPABILITIES = ['i2c', 'spi', 'onewire', 'serial', 'usb_otg'] as const
const IF_SETTINGS = [
  'enableI2C',
  'enableSPI',
  'enable1Wire',
  'enableSerial',
  'enableUsbGadget',
] as const

const ask = (method: string, ...args: unknown[]) => call<boolean | null>(method, ...args)

function setSource(entry: OsEntry, category: string) {
  return call('setSrc', {
    url: entry.url ?? '',
    downloadLen: entry.image_download_size ?? 0,
    extractLen: entry.extract_size ?? 0,
    expectedHash: entry.extract_sha256 ?? '',
    multipleFiles: entry.contains_multiple_files ?? false,
    parentCategory: category,
    osName: entry.name,
    initFormat: entry.init_format ?? '',
    releaseDate: entry.release_date ?? '',
    bmapUrl: entry.bmap_url ?? '',
  })
}

/** Erase: no customisation and no software capabilities. */
export async function selectErase(
  wizard: SelectionWizard,
  entry: OsEntry,
  name: string,
  category: string,
) {
  await setSource({ ...entry, release_date: '', bmap_url: '' }, category)
  await call('setSWCapabilitiesList', [])
  const [customization, forced] = await Promise.all([
    ask('imageSupportsCustomization'),
    ask('isSecureBootForcedByCliFlag'),
  ])
  wizard.setSelectedOsName(name)
  wizard.update({
    customizationSupported: Boolean(customization),
    piConnectAvailable: false,
    secureBootAvailable: Boolean(forced),
    passwordlessSudoAvailable: false,
    ccRpiAvailable: false,
    ifAndFeaturesAvailable: false,
    autoinstall: false,
  })
}

async function capabilityFlags(): Promise<Partial<WizardFlags>> {
  const [customization, connect, secure, forced, sudo, ccRpi, autoinstall] = await Promise.all([
    ask('imageSupportsCustomization'),
    ask('checkSWCapability', 'rpi_connect'),
    ask('checkSWCapability', 'secure_boot'),
    ask('isSecureBootForcedByCliFlag'),
    ask('checkSWCapability', 'passwordless_sudo'),
    ask('imageSupportsCcRpi'),
    ask('imageUsesAutoinstall'),
  ])
  const features = ccRpi
    ? (await Promise.all(IF_CAPABILITIES.map((c) => ask('checkHWAndSWCapability', c)))).some(
        Boolean,
      )
    : false
  return {
    customizationSupported: Boolean(customization),
    piConnectAvailable: Boolean(connect),
    secureBootAvailable: Boolean(secure) || Boolean(forced),
    passwordlessSudoAvailable: Boolean(sudo),
    ccRpiAvailable: Boolean(ccRpi),
    ifAndFeaturesAvailable: features,
    autoinstall: Boolean(autoinstall),
  }
}

function scrubSettings(wizard: SelectionWizard, flags: Partial<WizardFlags>) {
  if (!flags.piConnectAvailable) wizard.removeCustomizationSetting('piConnectEnabled')
  if (!flags.secureBootAvailable) wizard.removeCustomizationSetting('secureBootEnabled')
  if (!flags.passwordlessSudoAvailable) wizard.removeCustomizationSetting('passwordlessSudo')
  IF_SETTINGS.forEach((key) => {
    wizard.removeCustomizationSetting(key)
    void call('removePersistedCustomisationSetting', key)
  })
}

/** A concrete OS image from the list. */
export async function selectOs(wizard: SelectionWizard, entry: OsEntry, category: string) {
  await setSource(entry, category)
  await call('setSWCapabilitiesList', [...(entry.capabilities ?? [])])
  wizard.setSelectedOsName(entry.name)
  const flags = await capabilityFlags()
  wizard.update({
    ...flags,
    ...(flags.piConnectAvailable ? {} : { piConnectEnabled: false }),
    ...(flags.secureBootAvailable ? {} : { secureBootEnabled: false }),
    ifI2cEnabled: false,
    ifSpiEnabled: false,
    if1WireEnabled: false,
    ifSerial: 'Disabled',
    featUsbGadgetEnabled: false,
  })
  scrubSettings(wizard, flags)
}
