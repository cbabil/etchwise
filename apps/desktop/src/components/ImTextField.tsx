/**
 * Upstream `ImTextField.qml`: Qt Material 3 "Dense" outlined text field
 * (14px Roboto, focused outline in Qt Material's default pink accent).
 * Cut/Copy/Paste/Select All come from the webview's native context menu.
 */
import { TextField, type TextFieldProps } from '@mui/material'
import type { SxProps, Theme } from '@mui/material/styles'

import { colors, fontFamily, fontSize } from '@/theme/tokens'

/** Qt Quick Controls Material default accent (Material.Pink). */
export const materialAccent = '#e91e63'

export const textFieldSx: SxProps<Theme> = {
  '& .MuiOutlinedInput-root': {
    fontFamily,
    fontSize: fontSize.sm,
    backgroundColor: colors.mainBackground,
    '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
      borderColor: materialAccent,
      borderWidth: 2,
    },
    '&.Mui-error .MuiOutlinedInput-notchedOutline': { borderColor: colors.formLabelError },
  },
  '& .MuiOutlinedInput-input': { py: '10px', px: '12px' },
}

export type ImTextFieldProps = Omit<TextFieldProps, 'variant' | 'size'>

export function ImTextField({ sx, ...rest }: ImTextFieldProps) {
  return (
    <TextField
      variant="outlined"
      size="small"
      autoComplete="off"
      {...rest}
      sx={[textFieldSx, ...(Array.isArray(sx) ? sx : sx ? [sx] : [])] as SxProps<Theme>}
    />
  )
}
