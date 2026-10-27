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

/** Upstream 2.0.11 (38087f8c): pasting can bring control characters into a single-line field; drop them. */
// eslint-disable-next-line no-control-regex -- matching control characters is the point
const withoutControlChars = (text: string): string => text.replace(/[\u0000-\u001f\u007f]/g, '')

type ChangeHandler = NonNullable<TextFieldProps['onChange']>

/** `onChange` that sees the field's text with control characters removed. */
function scrubbedChange(onChange: ChangeHandler | undefined): ChangeHandler | undefined {
  if (!onChange) return undefined
  return (event) => {
    const input = event.target as HTMLInputElement
    const text = input.value
    const clean = withoutControlChars(text)
    if (clean !== text) {
      // Keep the caret where it was, less the characters removed before it
      const caret = withoutControlChars(text.slice(0, input.selectionStart ?? text.length)).length
      input.value = clean
      input.setSelectionRange?.(caret, caret)
    }
    onChange(event)
  }
}

export function ImTextField({ sx, onChange, ...rest }: ImTextFieldProps) {
  return (
    <TextField
      variant="outlined"
      size="small"
      autoComplete="off"
      {...rest}
      onChange={scrubbedChange(onChange)}
      sx={[textFieldSx, ...(Array.isArray(sx) ? sx : sx ? [sx] : [])] as SxProps<Theme>}
    />
  )
}
