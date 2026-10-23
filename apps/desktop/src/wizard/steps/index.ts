/**
 * Step registry: wizard step index (see `stepIds.ts`) to its component.
 * Step agents replace the component files, not this map.
 */
import type { ComponentType } from 'react'

import { Step } from '../stepIds'
import { DeviceSelectionStep } from './DeviceSelectionStep'
import { DoneStep } from './DoneStep'
import { HostnameCustomizationStep } from './HostnameCustomizationStep'
import { IfAndFeaturesCustomizationStep } from './IfAndFeaturesCustomizationStep'
import { LanguageSelectionStep } from './LanguageSelectionStep'
import { LocaleCustomizationStep } from './LocaleCustomizationStep'
import { OSSelectionStep } from './OSSelectionStep'
import { PiConnectCustomizationStep } from './PiConnectCustomizationStep'
import { RemoteAccessStep } from './RemoteAccessStep'
import { SecureBootCustomizationStep } from './SecureBootCustomizationStep'
import { StorageSelectionStep } from './StorageSelectionStep'
import { UserCustomizationStep } from './UserCustomizationStep'
import { WifiCustomizationStep } from './WifiCustomizationStep'
import { WritingStep } from './WritingStep'

export const stepComponents: Readonly<Record<number, ComponentType>> = {
  [Step.Language]: LanguageSelectionStep,
  [Step.Device]: DeviceSelectionStep,
  [Step.OS]: OSSelectionStep,
  [Step.Storage]: StorageSelectionStep,
  [Step.Hostname]: HostnameCustomizationStep,
  [Step.Locale]: LocaleCustomizationStep,
  [Step.User]: UserCustomizationStep,
  [Step.Wifi]: WifiCustomizationStep,
  [Step.RemoteAccess]: RemoteAccessStep,
  [Step.SecureBoot]: SecureBootCustomizationStep,
  [Step.PiConnect]: PiConnectCustomizationStep,
  [Step.IfAndFeatures]: IfAndFeaturesCustomizationStep,
  [Step.Writing]: WritingStep,
  [Step.Done]: DoneStep,
}
