/**
 * Upstream `UserCustomizationStep.qml`: username, password and
 * confirmation in a two-column grid, a help line, and (when the OS
 * supports it) the passwordless sudo option. A saved password hash is
 * kept when both password fields are left blank.
 */
import { ImPasswordField } from '@/components/ImPasswordField'
import { ImTextField } from '@/components/ImTextField'
import { useTr } from '@/i18n'

import { StepBase } from '../StepBase'
import { useWizard } from '../WizardContext'
import {
  DescriptionText,
  FormGrid,
  FormLabel,
  SectionContainer,
  StepContent,
} from './custom/layout'
import { PasswordlessSudoRow } from './custom/PasswordlessSudoRow'
import { useCustomStepNav } from './custom/useCustomStepNav'
import { useUserForm } from './custom/useUserForm'

type Form = ReturnType<typeof useUserForm>

function PasswordRows({ form }: { form: Form }) {
  const tr = useTr('UserCustomizationStep')
  const common = useTr('CommonStrings')
  const saved = form.hasSaved
  return (
    <>
      <FormLabel
        htmlFor="user-password"
        description={
          saved
            ? tr(
                'Enter a new password for this account, or leave blank to keep the previously saved password.',
              )
            : tr(
                'Enter a password for this account. You will need to re-enter it in the next field to confirm.',
              )
        }
      >
        {common('Password:')}
      </FormLabel>
      <ImPasswordField
        id="user-password"
        fullWidth
        placeholder={saved ? tr('Saved (hidden) — leave blank to keep') : tr('Enter password')}
        value={form.fields.password}
        onChange={(password) => form.set({ password })}
      />
      <FormLabel
        htmlFor="user-password-confirm"
        description={
          saved
            ? tr(
                'Re-enter the new password to confirm, or leave blank to keep the previously saved password.',
              )
            : tr('Re-enter the password to confirm it matches.')
        }
      >
        {tr('Confirm password:')}
      </FormLabel>
      <ImPasswordField
        id="user-password-confirm"
        fullWidth
        placeholder={saved ? tr('Re-enter to change password') : tr('Re-enter password')}
        value={form.fields.confirm}
        onChange={(confirm) => form.set({ confirm })}
      />
    </>
  )
}

function UserFields({ form }: { form: Form }) {
  const tr = useTr('UserCustomizationStep')
  const wizard = useWizard()
  return (
    <SectionContainer>
      <FormGrid>
        <FormLabel
          htmlFor="user-name"
          description={tr(
            'Enter a username for your Raspberry Pi account. The username must be lowercase and contain only letters, numbers, underscores, and hyphens.',
          )}
        >
          {tr('Username:')}
        </FormLabel>
        <ImTextField
          id="user-name"
          fullWidth
          placeholder={tr('Enter your username')}
          value={form.fields.username}
          onChange={(e) => form.setUsername(e.target.value)}
        />
        <PasswordRows form={form} />
      </FormGrid>
      <DescriptionText>
        {tr(
          'The username must be lowercase and contain only letters, numbers, underscores, and hyphens.',
        )}
      </DescriptionText>
      {wizard.passwordlessSudoAvailable ? (
        <PasswordlessSudoRow
          checked={form.fields.passwordlessSudo}
          onChange={(passwordlessSudo) => form.set({ passwordlessSudo })}
        />
      ) : null}
    </SectionContainer>
  )
}

export function UserCustomizationStep() {
  const tr = useTr('UserCustomizationStep')
  const form = useUserForm()
  const nav = useCustomStepNav(
    'UserCustomizationStep',
    'Save user account settings and continue to next customisation step',
  )
  return (
    <StepBase
      title={tr('Customisation: Choose username')}
      subtitle={tr('Create a user account for your Raspberry Pi')}
      {...nav}
      nextEnabled={form.nextEnabled}
      onNext={() => void form.submit()}
    >
      <StepContent>
        <UserFields form={form} />
      </StepContent>
    </StepBase>
  )
}
