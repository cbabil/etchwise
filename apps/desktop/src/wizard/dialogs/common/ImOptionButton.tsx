/**
 * Upstream `ImOptionButton.qml`: a bold 14px label on the left and a
 * white button on the right; only the button is clickable. Its
 * accessible name is "Label - Button, description".
 */
import { Box, Stack } from '@mui/material'

import { ImButton } from '@/components/ImButton'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

export interface ImOptionButtonProps {
  label: string
  buttonText: string
  onClick: () => void
  accessibleDescription?: string
  disabled?: boolean
}

export function ImOptionButton(props: ImOptionButtonProps) {
  const { label, buttonText, onClick, accessibleDescription, disabled = false } = props
  const name = `${label} - ${buttonText}`
  return (
    <Stack
      direction="row"
      spacing={`${spacing.medium}px`}
      sx={{ alignItems: 'center', minHeight: Math.max(sizes.buttonHeight - 8, 28) }}
    >
      <Box
        aria-hidden
        sx={{
          flex: 1,
          minWidth: 0,
          fontSize: fontSize.sm,
          fontWeight: 700,
          color: disabled ? colors.textDescription : colors.formLabel,
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
        }}
      >
        {label}
      </Box>
      <ImButton
        onClick={onClick}
        disabled={disabled}
        aria-label={accessibleDescription ? `${name}, ${accessibleDescription}` : name}
      >
        {buttonText}
      </ImButton>
    </Stack>
  )
}
