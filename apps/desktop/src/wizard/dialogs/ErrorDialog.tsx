/**
 * main.qml's error dialog: opens on the backend `error(message)` signal
 * with an "Error" heading, the message and a Continue button.
 */
import { useEffect, useState } from 'react'

import { onImagerEvent } from '@/api/imager'
import { BaseDialog } from '@/components/BaseDialog'
import { DialogBody, DialogButtons, DialogHeading } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { useTr } from '@/i18n'

/** Plain text of upstream's StyledText message. */
const stripTags = (html: string): string =>
  html.replace(/<br\s*\/?>/gi, '\n').replace(/<[^>]+>/g, '')

export function ErrorDialog() {
  const tr = useTr('main')
  const common = useTr('CommonStrings')
  const [message, setMessage] = useState<string | null>(null)
  useEffect(() => {
    let off: (() => void) | null = null
    let live = true
    void onImagerEvent((event) => {
      if (event.name === 'error') setMessage(String(event.data ?? ''))
    }).then((unsubscribe) => {
      if (live) off = unsubscribe
      else unsubscribe()
    })
    return () => {
      live = false
      off?.()
    }
  }, [])
  const close = () => setMessage(null)
  return (
    <BaseDialog open={message !== null} onClose={close} title={tr('Error')}>
      <DialogHeading>{tr('Error')}</DialogHeading>
      <DialogBody>
        <span style={{ whiteSpace: 'pre-line' }}>{stripTags(message ?? '')}</span>
      </DialogBody>
      <DialogButtons>
        <ImButton onClick={close} accessibleDescription={tr('Close the error dialog and continue')}>
          {common('Continue')}
        </ImButton>
      </DialogButtons>
    </BaseDialog>
  )
}
