/**
 * Size fields of Kura's disk layout: whole GiB typed as digits, and a fixed
 * row (a caption, then its size) for the root, /tmp and swap.
 */
import { Box } from '@mui/material'

import { ImTextField } from '@/components/ImTextField'

import { FormLabel, FormRow } from '../custom/layout'
import { acceptsInput } from '../custom/validation'

const SIZE_RE = /^\d{1,4}$/
/** Width of the first column: captions and folder pickers. */
export const FIRST_COLUMN = 180

export interface SizeFieldProps {
  label: string
  value: string
  disabled: boolean
  onChange: (value: string) => void
}

export function SizeField({ label, value, disabled, onChange }: SizeFieldProps) {
  return (
    <ImTextField
      value={value}
      disabled={disabled}
      onChange={(e) => acceptsInput(SIZE_RE, e.target.value) && onChange(e.target.value)}
      slotProps={{ htmlInput: { 'aria-label': label, inputMode: 'numeric' } }}
      sx={{ width: 96 }}
    />
  )
}

/** A fixed row: a caption, then its size. */
export function SizeRow(props: SizeFieldProps & { caption: string }) {
  return (
    <FormRow>
      <Box sx={{ width: FIRST_COLUMN }}>
        <FormLabel isDisabled={props.disabled}>{props.caption}</FormLabel>
      </Box>
      <SizeField {...props} />
    </FormRow>
  )
}
