/**
 * The navigation and mutation functions `useWizard()` exposes. State lives
 * in a ref mirrored into React state so successive calls in one handler
 * see each other's results.
 */
import { type RefObject, useMemo } from 'react'

import { call } from '@/api/imager'
import { sizes } from '@/theme/tokens'

import { advance, isPermissible, selectDevice, selectOs } from './navigation'
import { jump, resetToWriteStep, resetWizard, retreat, skipCustomisation } from './operations'
import type { CustomizationSettings, WizardFlags, WizardState } from './state'

export interface WizardActions {
  next: () => void
  back: () => void
  jumpTo: (step: number) => void
  skipCustomisation: (extra?: Partial<WizardFlags>) => void
  update: (patch: Partial<WizardFlags>) => void
  setSelectedDeviceName: (name: string) => void
  setSelectedOsName: (name: string) => void
  setSelectedStorageName: (name: string) => void
  setCustomizationSettings: (settings: CustomizationSettings) => void
  updateCustomizationSettings: (patch: CustomizationSettings) => void
  removeCustomizationSetting: (key: string) => void
  isStepPermissible: (step: number) => boolean
  resetWizard: () => void
  resetToWriteStep: () => void
  setSidebarWidth: (width: number, persist?: boolean) => void
}

export type Commit = (change: (state: WizardState) => WizardState) => void

export const clampSidebarWidth = (width: number): number =>
  Math.max(sizes.sidebarMinWidth, Math.min(sizes.sidebarMaxWidth, width))

function settingsActions(commit: Commit) {
  return {
    setCustomizationSettings: (settings: CustomizationSettings) =>
      commit((s) => ({ ...s, customizationSettings: { ...settings } })),
    updateCustomizationSettings: (patch: CustomizationSettings) =>
      commit((s) => ({ ...s, customizationSettings: { ...s.customizationSettings, ...patch } })),
    removeCustomizationSetting: (key: string) =>
      commit((s) => ({
        ...s,
        customizationSettings: Object.fromEntries(
          Object.entries(s.customizationSettings).filter(([k]) => k !== key),
        ),
      })),
  }
}

function resetActions(commit: Commit) {
  return {
    resetWizard: () => {
      commit(resetWizard)
      void call('setHWIndex', -1)
      void call('setSrc', { url: '' })
      void call('setDst', '', 0)
    },
    resetToWriteStep: () => {
      commit(resetToWriteStep)
      void call('setDst', '', 0)
    },
    setSidebarWidth: (width: number, persist = false) => {
      const clamped = clampSidebarWidth(width)
      commit((s) => ({ ...s, sidebarWidth: clamped }))
      if (persist) void call('setSetting', 'sidebarWidth', String(clamped))
    },
  }
}

export function useWizardActions(ref: RefObject<WizardState>, commit: Commit): WizardActions {
  return useMemo(
    () => ({
      next: () => {
        const { state, apply } = advance(ref.current)
        if (apply) void call('applyCustomisationFromSettings', state.customizationSettings)
        commit(() => state)
      },
      back: () => commit(retreat),
      jumpTo: (step: number) => commit((s) => jump(s, step)),
      skipCustomisation: (extra?: Partial<WizardFlags>) =>
        commit((s) => skipCustomisation(s, extra)),
      update: (patch: Partial<WizardFlags>) => commit((s) => ({ ...s, ...patch })),
      setSelectedDeviceName: (name: string) => commit((s) => selectDevice(s, name)),
      setSelectedOsName: (name: string) => commit((s) => selectOs(s, name)),
      setSelectedStorageName: (name: string) =>
        commit((s) => ({ ...s, selectedStorageName: name })),
      isStepPermissible: (step: number) => isPermissible(ref.current.permissibleStepsBitmap, step),
      ...settingsActions(commit),
      ...resetActions(commit),
    }),
    [ref, commit],
  )
}
