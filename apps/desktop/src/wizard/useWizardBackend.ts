/**
 * Start-up and signal handling of upstream `WizardContainer.qml`: loads
 * saved customisation, the secure-boot key flag and the sidebar width,
 * picks the first step, and reacts to `osListUnavailableChanged` and
 * `cancelled`.
 */
import { useEffect } from 'react'

import { call, onImagerEvent } from '@/api/imager'

import { withStep } from './navigation'
import { jump } from './operations'
import type { CustomizationSettings, WizardState } from './state'
import { Step } from './stepIds'
import { clampSidebarWidth, type Commit } from './useWizardActions'

async function loadInitial(showLanguageSelection: boolean | undefined) {
  const [settings, rsaKey, width, unavailable, embedded, showLanguage] = await Promise.all([
    call<CustomizationSettings | null>('getSavedCustomisationSettings'),
    call<string | null>('getStringSetting', 'secureboot_rsa_key'),
    call<string | null>('getStringSetting', 'sidebarWidth'),
    call<boolean | null>('isOsListUnavailable'),
    call<boolean | null>('isEmbeddedMode'),
    showLanguageSelection ?? call<boolean | null>('showLanguageSelection'),
  ])
  return (state: WizardState): WizardState => {
    const online = !unavailable
    const first = showLanguage ? Step.Language : online ? Step.Device : Step.OS
    const parsed = parseInt(width ?? '', 10)
    return withStep(
      {
        ...state,
        hasNetworkConnectivity: online,
        showLanguageSelection: Boolean(showLanguage),
        customizationSettings: { ...(settings ?? {}) },
        secureBootKeyConfigured: Boolean(rsaKey),
        disableWarnings: Boolean(embedded),
        sidebarWidth: Number.isNaN(parsed) ? state.sidebarWidth : clampSidebarWidth(parsed),
      },
      first,
    )
  }
}

async function onOsListChanged(commit: Commit) {
  const unavailable = await call<boolean | null>('isOsListUnavailable')
  commit((s) => {
    const online = !unavailable
    const updated = { ...s, hasNetworkConnectivity: online }
    return online && s.currentStep === Step.OS && !s.isWriting
      ? jump(updated, Step.Device)
      : updated
  })
}

export function useWizardBackend(
  commit: Commit,
  setReady: (ready: boolean) => void,
  showLanguageSelection?: boolean,
) {
  useEffect(() => {
    let live = true
    void loadInitial(showLanguageSelection).then((apply) => {
      if (!live) return
      commit(apply)
      setReady(true)
    })
    return () => {
      live = false
    }
  }, [commit, setReady, showLanguageSelection])

  useEffect(() => {
    let unsubscribe: (() => void) | null = null
    let live = true
    void onImagerEvent((event) => {
      if (event.name === 'osListUnavailableChanged') void onOsListChanged(commit)
      if (event.name === 'cancelled')
        commit((s) => (s.currentStep !== Step.Writing ? jump(s, Step.Writing) : s))
    }).then((off) => {
      if (live) unsubscribe = off
      else off()
    })
    return () => {
      live = false
      unsubscribe?.()
    }
  }, [commit])
}
