/**
 * Settings plumbing shared by the customisation steps: runtime edits of
 * `customizationSettings` (upstream `wizardContainer.customizationSettings.x
 * = …` / `delete …`) and the two upstream persistence styles
 * (`setPersistedCustomisationSetting` per key, or read-modify-write of the
 * whole saved map).
 */
import { call } from '@/api/imager'

import type { CustomizationSettings } from '../../state'
import type { WizardApi } from '../../WizardContext'

type SettingsApi = Pick<WizardApi, 'updateCustomizationSettings' | 'removeCustomizationSetting'>

/** Sets `patch` and deletes `remove` in the runtime customisation settings. */
export function editSettings(
  wizard: SettingsApi,
  patch: CustomizationSettings,
  remove: readonly string[] = [],
): void {
  if (Object.keys(patch).length > 0) wizard.updateCustomizationSettings(patch)
  remove.forEach((key) => wizard.removeCustomizationSetting(key))
}

/** `imageWriter.setPersistedCustomisationSetting(key, value)`. */
export function persist(key: string, value: unknown): void {
  void call('setPersistedCustomisationSetting', key, value)
}

/** `imageWriter.removePersistedCustomisationSetting(key)`. */
export function unpersist(key: string): void {
  void call('removePersistedCustomisationSetting', key)
}

/**
 * `saved = getSavedCustomisationSettings(); …; setSavedCustomisationSettings(saved)`
 * with `patch` applied and `remove` deleted.
 */
export async function rewriteSaved(
  patch: CustomizationSettings,
  remove: readonly string[],
): Promise<void> {
  const saved = (await call<CustomizationSettings | null>('getSavedCustomisationSettings')) ?? {}
  const kept = Object.fromEntries(Object.entries(saved).filter(([key]) => !remove.includes(key)))
  await call('setSavedCustomisationSettings', { ...kept, ...patch })
}

/** A setting as a string ('' when absent or not a string). */
export function str(value: unknown): string {
  return typeof value === 'string' ? value : ''
}

/** Upstream `x === true || x === "true"`. */
export function isTrue(value: unknown): boolean {
  return value === true || value === 'true'
}
