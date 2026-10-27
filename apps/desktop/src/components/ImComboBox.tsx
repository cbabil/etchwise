/**
 * Upstream `ImComboBox.qml`: a non-editable drop-down with a search field
 * in the open popup, wheel / arrow stepping while closed, and
 * Enter/Return/Space to open.
 */
import { ArrowDropDown } from '@mui/icons-material'
import { Box, Popover } from '@mui/material'
import { type KeyboardEvent, useState, type WheelEvent } from 'react'

import { colors, fontFamily, fontSize, sizes } from '@/theme/tokens'

import { popupHeight } from './comboFilter'
import { ComboPopupList } from './ComboPopupList'
import { materialAccent } from './ImTextField'
import { type ComboPopup, useComboPopup } from './useComboPopup'

export interface ImComboBoxProps {
  options: readonly string[]
  /** Selected index, -1 for none (upstream `currentIndex`). */
  value: number
  /** Upstream `activated(index)`. */
  onChange: (index: number) => void
  disabled?: boolean
  indicateError?: boolean
  ariaLabel?: string
}

const fieldSx = (disabled: boolean, indicateError: boolean) => ({
  height: sizes.buttonHeight,
  pl: '12px',
  pr: '4px',
  display: 'flex',
  alignItems: 'center',
  fontFamily,
  fontSize: fontSize.sm,
  color: disabled ? colors.formLabelDisabled : colors.formLabel,
  backgroundColor: colors.mainBackground,
  border: `1px solid ${indicateError ? colors.formLabelError : 'rgba(0, 0, 0, 0.38)'}`,
  borderRadius: `${sizes.buttonRadius}px`,
  outline: 'none',
  cursor: 'default',
  '&:focus-visible': {
    borderColor: materialAccent,
    boxShadow: `inset 0 0 0 1px ${materialAccent}`,
  },
})

function useClosedKeys(props: ImComboBoxProps, popup: ComboPopup) {
  const { options, value, onChange, disabled = false } = props
  const step = (delta: number) => {
    const next = value + delta
    if (next >= 0 && next < options.length) onChange(next)
  }
  const onKeyDown = (event: KeyboardEvent) => {
    if (disabled) return
    if (event.key === 'Enter' || event.key === ' ') popup.openPopup()
    else if (event.key === 'ArrowDown') step(1)
    else if (event.key === 'ArrowUp') step(-1)
    else return
    event.preventDefault()
  }
  const onWheel = (event: WheelEvent) => {
    if (!disabled && !popup.open && event.deltaY !== 0) step(event.deltaY > 0 ? 1 : -1)
  }
  return { onKeyDown, onWheel }
}

function ComboPopover({
  anchor,
  popup,
  props,
}: {
  anchor: HTMLElement | null
  popup: ComboPopup
  props: ImComboBoxProps
}) {
  const rect = anchor?.getBoundingClientRect()
  const available = rect ? window.innerHeight - rect.bottom - 8 : 300
  return (
    <Popover
      open={popup.open && anchor !== null}
      anchorEl={anchor}
      onClose={popup.close}
      anchorOrigin={{ vertical: 'bottom', horizontal: 'left' }}
      transitionDuration={0}
      slotProps={{
        paper: {
          elevation: 0,
          // On the whole popup: clicking the list or its scrollbar moves
          // focus off the search field, and the keys must still work
          onKeyDown: popup.onKeyDown,
          sx: {
            width: rect?.width || 200,
            height: popupHeight(props.options.length, available),
            position: 'relative',
            border: `1px solid ${colors.popupBorder}`,
            borderRadius: `${sizes.sectionBorderRadius}px`,
            overflow: 'hidden',
          },
        },
      }}
    >
      <ComboPopupList
        filtered={popup.filtered}
        total={props.options.length}
        search={popup.search}
        highlighted={popup.highlighted}
        indicateError={props.indicateError ?? false}
        onPick={popup.pick}
        onSearch={popup.setSearchText}
      />
    </Popover>
  )
}

export function ImComboBox(props: ImComboBoxProps) {
  const { options, value, disabled = false, indicateError = false, ariaLabel } = props
  const [anchor, setAnchor] = useState<HTMLDivElement | null>(null)
  const popup = useComboPopup(options, value, props.onChange)
  const keys = useClosedKeys(props, popup)
  return (
    <>
      <Box
        ref={setAnchor}
        role="combobox"
        tabIndex={disabled ? -1 : 0}
        aria-expanded={popup.open}
        aria-disabled={disabled}
        aria-label={ariaLabel ?? options[value] ?? ''}
        {...(indicateError ? { 'aria-invalid': true } : {})}
        onClick={() => !disabled && popup.openPopup()}
        {...keys}
        sx={fieldSx(disabled, indicateError)}
      >
        <Box
          component="span"
          sx={{
            flex: 1,
            minWidth: 0,
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
        >
          {options[value] ?? ''}
        </Box>
        <ArrowDropDown sx={{ color: colors.textMetadata }} />
      </Box>
      <ComboPopover anchor={anchor} popup={popup} props={props} />
    </>
  )
}
