/** One key row of upstream `SshKeyManager.qml`: shortened monospace key text and REMOVE. */
import { Box } from '@mui/material'

import { ImButton } from '@/components/ImButton'
import { useTr } from '@/i18n'
import { colors, fontSize, spacing } from '@/theme/tokens'

import { FormRow } from './layout'
import { displayKey, keyAccessibleName, removeDescription } from './sshKeys'

export interface SshKeyRowProps {
  sshKey: string
  index: number
  disabled: boolean
  onRemove: () => void
}

export function SshKeyRow({ sshKey, index, disabled, onRemove }: SshKeyRowProps) {
  const tr = useTr('SshKeyManager')
  return (
    <Box role="group" aria-label={keyAccessibleName(tr, sshKey, index)}>
      <FormRow gap={spacing.small}>
        <Box
          aria-hidden
          sx={{
            flex: 1,
            minWidth: 0,
            fontFamily: 'monospace',
            fontSize: fontSize.sm,
            color: colors.formLabel,
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
        >
          {displayKey(sshKey)}
        </Box>
        <ImButton
          sx={{ minWidth: 80 }}
          disabled={disabled}
          accessibleDescription={removeDescription(tr, sshKey, index)}
          onClick={onRemove}
        >
          {tr('Remove')}
        </ImButton>
      </FormRow>
    </Box>
  )
}
