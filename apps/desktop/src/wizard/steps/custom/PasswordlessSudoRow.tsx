/**
 * The passwordless sudo row of upstream `UserCustomizationStep.qml`:
 * checking the box opens the warning dialog, and cancelling it unchecks
 * the box again. Shown only when the OS supports passwordless sudo.
 */
import { useState } from 'react'

import { ImCheckBox } from '@/components/ImCheckBox'
import { useTr } from '@/i18n'
import { spacing } from '@/theme/tokens'

import { PasswordlessSudoWarningDialog } from '../../dialogs/PasswordlessSudoWarningDialog'
import { FormRow, InfoIcon } from './layout'

export interface PasswordlessSudoRowProps {
  checked: boolean
  onChange: (checked: boolean) => void
}

export function PasswordlessSudoRow({ checked, onChange }: PasswordlessSudoRowProps) {
  const tr = useTr('UserCustomizationStep')
  const [asking, setAsking] = useState(false)
  const info = tr(
    'Allows any process running as this user to gain full root privileges without a password. Only enable this if you have a specific need, such as automated scripts or headless operation.',
  )
  return (
    <FormRow gap={spacing.small}>
      <ImCheckBox
        label={tr('Enable passwordless sudo')}
        checked={checked}
        accessibleDescription={tr(
          'Allow this user to run sudo commands without entering a password.',
        )}
        onChange={(value) => {
          onChange(value)
          if (value) setAsking(true)
        }}
      />
      <InfoIcon tooltip={info} label={tr('Passwordless sudo information: ') + info} cursor="help" />
      <PasswordlessSudoWarningDialog
        open={asking}
        onConfirm={() => setAsking(false)}
        onCancel={() => {
          setAsking(false)
          onChange(false)
        }}
      />
    </FormRow>
  )
}
