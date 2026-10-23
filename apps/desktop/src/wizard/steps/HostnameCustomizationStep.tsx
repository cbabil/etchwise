/**
 * Upstream `HostnameCustomizationStep.qml`: one input-restricted hostname
 * field prefilled from the saved settings; Next stores the trimmed
 * hostname (or removes it when empty) at runtime and persistently.
 */
import { useState } from 'react'

import { ImTextField } from '@/components/ImTextField'
import { useTr } from '@/i18n'

import { StepBase } from '../StepBase'
import { useWizard, type WizardApi } from '../WizardContext'
import { DescriptionText, SectionContainer, StepContent } from './custom/layout'
import { editSettings, persist, str, unpersist } from './custom/settings'
import { useCustomStepNav } from './custom/useCustomStepNav'
import { useMountEffect } from './custom/useMountEffect'
import { acceptsInput, HOSTNAME_RE } from './custom/validation'

/** Upstream `onNextClicked`. */
export function saveHostname(wizard: WizardApi, text: string): void {
  const hostname = text.trim()
  if (hostname.length > 0) {
    editSettings(wizard, { hostname })
    wizard.update({ hostnameConfigured: true })
    persist('hostname', hostname)
  } else {
    editSettings(wizard, {}, ['hostname'])
    wizard.update({ hostnameConfigured: false })
    unpersist('hostname')
  }
}

export function HostnameCustomizationStep() {
  const tr = useTr('HostnameCustomizationStep')
  const wizard = useWizard()
  const nav = useCustomStepNav(
    'HostnameCustomizationStep',
    'Save hostname and continue to next customisation step',
  )
  const [hostname, setHostname] = useState(() => str(wizard.customizationSettings.hostname))
  useMountEffect(() => {
    if (str(wizard.customizationSettings.hostname)) wizard.update({ hostnameConfigured: true })
  })
  const help = tr(
    'A hostname is a unique name that identifies your Raspberry Pi on the network. It should contain only letters, numbers, and hyphens.',
  )
  const placeholder = tr('Enter your hostname')
  return (
    <StepBase
      title={tr('Customisation: Choose hostname')}
      {...nav}
      onNext={() => {
        saveHostname(wizard, hostname)
        wizard.next()
      }}
    >
      <StepContent>
        <SectionContainer>
          <ImTextField
            fullWidth
            placeholder={placeholder}
            value={hostname}
            onChange={(e) =>
              acceptsInput(HOSTNAME_RE, e.target.value) && setHostname(e.target.value)
            }
            slotProps={{ htmlInput: { 'aria-label': placeholder, 'aria-description': help } }}
          />
          <DescriptionText>{help}</DescriptionText>
        </SectionContainer>
      </StepContent>
    </StepBase>
  )
}
