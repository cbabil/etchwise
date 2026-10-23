/**
 * Upstream `ConfirmSystemDriveDialog.qml`: before a system drive can be
 * selected the user must type its exact name. Paste (keyboard, context
 * menu, middle click, drop) is blocked; Enter confirms once the name
 * matches; CANCEL (red) and Escape cancel.
 */
import { Box } from '@mui/material'
import { type ClipboardEvent, type KeyboardEvent, type SyntheticEvent, useState } from 'react'

import { BaseDialog } from '@/components/BaseDialog'
import { DialogBody, DialogButtons } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { ImTextField } from '@/components/ImTextField'
import { useTr } from '@/i18n'
import { colors, fontSize } from '@/theme/tokens'

export interface ConfirmSystemDriveDialogProps {
  open: boolean
  driveName: string
  sizeStr: string
  mountpoints: readonly string[]
  onConfirm: () => void
  onCancel: () => void
}

const Separator = () => <Box sx={{ height: '1px', backgroundColor: colors.titleSeparator }} />

const block = (event: SyntheticEvent) => event.preventDefault()

const isPasteKey = (e: KeyboardEvent) =>
  (e.key.toLowerCase() === 'v' && (e.ctrlKey || e.metaKey)) || (e.key === 'Insert' && e.shiftKey)

function Warning() {
  const common = useTr('CommonStrings')
  return (
    <DialogBody>
      {common(
        'Selecting the wrong drive will permanently erase data and can render your computer inoperable.',
      )}
      <br />
      <br />
      {common(
        'System drives typically contain files essential to the operation of your computer, and may include your personal files (photos, videos, documents).',
      )}
      <br />
      <br />
      {common('Only proceed if you fully understand the risks.')}
    </DialogBody>
  )
}

function DriveInfo({
  sizeStr,
  mountpoints,
}: Pick<ConfirmSystemDriveDialogProps, 'sizeStr' | 'mountpoints'>) {
  const tr = useTr('ConfirmSystemDriveDialog')
  const mounted = mountpoints.length > 0 ? mountpoints.join(', ') : tr('Not mounted')
  return (
    <Box role="group" aria-label={tr('Drive information')}>
      <DialogBody>{tr('Size: %1', sizeStr)}</DialogBody>
      <DialogBody>{tr('Mounted as: %1', mounted)}</DialogBody>
    </Box>
  )
}

function NameInput(props: {
  driveName: string
  value: string
  onChange: (v: string) => void
  onEnter: () => void
}) {
  const tr = useTr('ConfirmSystemDriveDialog')
  const placeholder = tr('Type drive name exactly as shown above')
  const onKeyDown = (event: KeyboardEvent) => {
    if (isPasteKey(event)) event.preventDefault()
    else if (event.key === 'Enter') props.onEnter()
  }
  return (
    <ImTextField
      fullWidth
      autoFocus
      value={props.value}
      placeholder={placeholder}
      onChange={(event) => props.onChange(event.target.value)}
      onKeyDown={onKeyDown}
      onPaste={(event: ClipboardEvent) => event.preventDefault()}
      onContextMenu={block}
      onDrop={block}
      onAuxClick={block}
      slotProps={{
        htmlInput: {
          'aria-label': tr('Drive name input. Type exactly: %1. %2', props.driveName, placeholder),
        },
      }}
    />
  )
}

function Buttons({
  canContinue,
  onConfirm,
  onCancel,
}: {
  canContinue: boolean
  onConfirm: () => void
  onCancel: () => void
}) {
  const tr = useTr('ConfirmSystemDriveDialog')
  return (
    <DialogButtons>
      <ImButtonRed
        onClick={onCancel}
        accessibleDescription={tr(
          'Cancel operation and return to storage selection to choose a different device',
        )}
      >
        {tr('CANCEL')}
      </ImButtonRed>
      <ImButton
        onClick={onConfirm}
        disabled={!canContinue}
        accessibleDescription={tr(
          'Proceed to write the image to this system drive after confirming the drive name',
        )}
      >
        {tr('CONTINUE')}
      </ImButton>
    </DialogButtons>
  )
}

export function ConfirmSystemDriveDialog(props: ConfirmSystemDriveDialogProps) {
  const tr = useTr('ConfirmSystemDriveDialog')
  const [text, setText] = useState('')
  const [wasOpen, setWasOpen] = useState(props.open)
  if (props.open !== wasOpen) {
    setWasOpen(props.open)
    if (props.open) setText('')
  }
  const canContinue = props.driveName.length > 0 && text === props.driveName
  const confirm = () => {
    if (canContinue) props.onConfirm()
  }
  return (
    <BaseDialog open={props.open} onClose={props.onCancel} title={props.driveName}>
      <Warning />
      <Separator />
      <DriveInfo sizeStr={props.sizeStr} mountpoints={props.mountpoints} />
      <Separator />
      <DialogBody>{tr('To continue, type the exact drive name below:')}</DialogBody>
      <Box sx={{ fontSize: fontSize.sm, fontWeight: 700, color: colors.textDescription }}>
        {props.driveName}
      </Box>
      <NameInput driveName={props.driveName} value={text} onChange={setText} onEnter={confirm} />
      <Buttons canContinue={canContinue} onConfirm={confirm} onCancel={props.onCancel} />
    </BaseDialog>
  )
}
