/** A two-button confirmation `BaseDialog` (heading, body, Cancel / red confirm). */
import type { ReactNode } from 'react'

import { BaseDialog } from '@/components/BaseDialog'
import { DialogButtons, DialogHeading } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'

import { InfoText } from './dialogParts'

export interface ConfirmDialogProps {
  open: boolean
  title: string
  body: ReactNode
  cancelDescription: string
  confirmLabel: string
  confirmDescription: string
  onCancel: () => void
  onConfirm: () => void
}

export function ConfirmDialog(props: ConfirmDialogProps) {
  const common = useTr('CommonStrings')
  return (
    <BaseDialog open={props.open} onClose={props.onCancel} title={props.title}>
      <DialogHeading>{props.title}</DialogHeading>
      <InfoText>{props.body}</InfoText>
      <DialogButtons>
        <ImButton onClick={props.onCancel} accessibleDescription={props.cancelDescription}>
          {common('Cancel')}
        </ImButton>
        <ImButtonRed onClick={props.onConfirm} accessibleDescription={props.confirmDescription}>
          {props.confirmLabel}
        </ImButtonRed>
      </DialogButtons>
    </BaseDialog>
  )
}

/** Renders Qt StyledText with `<b>` runs as bold spans (other tags dropped). */
export function styledText(html: string): ReactNode[] {
  return html
    .split(/(<b>.*?<\/b>)/g)
    .filter(Boolean)
    .map((part, i) =>
      part.startsWith('<b>') ? <b key={i}>{part.slice(3, -4)}</b> : part.replace(/<[^>]+>/g, ''),
    )
}
