/**
 * Upstream `LocaleCustomizationStep.qml`: capital city (with the Wi-Fi
 * regulatory-domain info icon), time zone and keyboard layout drop-downs.
 * Picking a city pre-fills the time zone and keyboard unless the user
 * changed them; Next stores all three and sets `localeConfigured`.
 */
import { Box } from '@mui/material'
import type { ReactNode } from 'react'

import { ImComboBox } from '@/components/ImComboBox'
import { useTr } from '@/i18n'

import { StepBase } from '../StepBase'
import { useWizard } from '../WizardContext'
import { FormLabel, FormRow, InfoIcon, SectionContainer, StepContent } from './custom/layout'
import { useCustomStepNav } from './custom/useCustomStepNav'
import { useLocaleForm } from './custom/useLocaleForm'

interface ComboRowProps {
  label: string
  description: string
  options: readonly string[]
  value: string
  onPick: (value: string) => void
  children?: ReactNode
}

function ComboRow({ label, description, options, value, onPick, children }: ComboRowProps) {
  return (
    <FormRow>
      <FormLabel description={description}>{label}</FormLabel>
      <Box sx={{ flex: 1, minWidth: 0 }}>
        <ImComboBox
          options={options}
          value={options.indexOf(value)}
          onChange={(index) => onPick(options[index] ?? '')}
          ariaLabel={value ? `${label} ${value}` : label}
        />
      </Box>
      {children}
    </FormRow>
  )
}

function LocaleFields({ form }: { form: ReturnType<typeof useLocaleForm> }) {
  const tr = useTr('LocaleCustomizationStep')
  return (
    <>
      <ComboRow
        label={tr('Capital city:')}
        description={tr(
          "Choose your nearest capital city. This will automatically recommend the correct time zone and keyboard layout for your region, and set the wireless regulatory domain for your country's Wi-Fi regulations.",
        )}
        options={form.lists.cities}
        value={form.city}
        onPick={(city) => void form.pickCity(city)}
      >
        <InfoIcon
          role="button"
          label={tr('Why am I being asked this?')}
          tooltip={tr('This also sets the Wi-Fi regulatory domain for your region.')}
        />
      </ComboRow>
      <ComboRow
        label={tr('Time zone:')}
        description={tr(
          'Choose your time zone so your Raspberry Pi displays the correct local time. This is automatically recommended based on your capital city selection, but you can change it if the suggestion is incorrect.',
        )}
        options={form.lists.timezones}
        value={form.timezone}
        onPick={form.pickTimezone}
      />
      <ComboRow
        label={tr('Keyboard layout:')}
        description={tr(
          'Choose your keyboard layout so keys produce the correct characters when typing. This is automatically recommended based on your capital city selection, but you can change it if you use a different keyboard layout.',
        )}
        options={form.lists.keymaps}
        value={form.keyboard}
        onPick={form.pickKeyboard}
      />
    </>
  )
}

export function LocaleCustomizationStep() {
  const tr = useTr('LocaleCustomizationStep')
  const wizard = useWizard()
  const form = useLocaleForm()
  const nav = useCustomStepNav(
    'LocaleCustomizationStep',
    'Save localisation settings and continue to next customisation step',
  )
  return (
    <StepBase
      title={tr('Customisation: Localisation')}
      subtitle={tr('Select your location for suggested time zone and keyboard layout')}
      {...nav}
      onNext={() => {
        form.save()
        wizard.next()
      }}
    >
      <StepContent>
        <SectionContainer>
          <LocaleFields form={form} />
        </SectionContainer>
      </StepContent>
    </StepBase>
  )
}
