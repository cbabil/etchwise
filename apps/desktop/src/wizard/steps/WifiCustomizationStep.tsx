/**
 * Upstream `WifiCustomizationStep.qml`: secure / open network, SSID
 * (saved or auto-detected), PSK with confirmation and inline validation,
 * hidden SSID. Leaving the SSID empty skips Wi-Fi; Next hashes the
 * passphrase and stores everything at runtime and in the saved settings.
 */
import { useState } from 'react'

import { useTr } from '@/i18n'

import { KeychainPermissionDialog } from '../dialogs/KeychainPermissionDialog'
import { StepBase } from '../StepBase'
import { useWizard } from '../WizardContext'
import { SectionContainer, StepContent } from './custom/layout'
import { saveWifi } from './custom/saveWifi'
import { useCustomStepNav } from './custom/useCustomStepNav'
import { useWifiForm } from './custom/useWifiForm'
import { useWifiPrefill } from './custom/useWifiPrefill'
import { WifiFields } from './custom/WifiFields'

export function WifiCustomizationStep() {
  const tr = useTr('WifiCustomizationStep')
  const wizard = useWizard()
  const form = useWifiForm()
  const prefill = useWifiPrefill(form)
  const [busy, setBusy] = useState(false)
  const nav = useCustomStepNav(
    'WifiCustomizationStep',
    'Save Wi-Fi settings and continue to next customisation step',
  )
  const onNext = async () => {
    setBusy(true)
    try {
      if (await saveWifi(wizard, form.fields)) wizard.next()
    } finally {
      setBusy(false)
    }
  }
  return (
    <StepBase
      title={tr('Customisation: Choose Wi‑Fi')}
      {...nav}
      nextEnabled={!busy && form.nextEnabled}
      onNext={() => void onNext()}
    >
      <StepContent>
        <SectionContainer>
          <WifiFields form={form} />
        </SectionContainer>
      </StepContent>
      <KeychainPermissionDialog open={prefill.keychainOpen} onResponse={prefill.respond} />
    </StepBase>
  )
}
