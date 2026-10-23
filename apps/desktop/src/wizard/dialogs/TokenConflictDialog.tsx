/**
 * `WizardContainer.qml` token conflict dialog: opens on the backend
 * `connectTokenConflictDetected(token)` signal; "Replace token" unlocks
 * after 1.5 s and overwrites the Raspberry Pi Connect token.
 */
import { useState } from 'react'

import { call } from '@/api/imager'
import { BaseDialog } from '@/components/BaseDialog'
import { DialogButtons, DialogHeading } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'
import { colors, fontSize } from '@/theme/tokens'

import { InfoText } from './common/dialogParts'
import { useDelayedFlag } from './common/useDelayedFlag'
import { useImagerEvent } from './common/useImagerEvent'

export const TOKEN_ACCEPT_DELAY_MS = 1500

export function TokenConflictDialog() {
  const tr = useTr('WizardContainer')
  const [token, setToken] = useState<string | null>(null)
  const open = token !== null
  const allowAccept = useDelayedFlag(open, TOKEN_ACCEPT_DELAY_MS)
  useImagerEvent('connectTokenConflictDetected', (data) => setToken(String(data ?? '')))
  const close = () => setToken(null)
  const replace = () => {
    if (token) void call('overwriteConnectToken', token)
    close()
  }
  const body =
    tr('A new Raspberry Pi Connect token was received that differs from your current one.\n\n') +
    tr('Do you want to overwrite the existing token?\n\n') +
    tr('Warning: Only overwrite the token if you initiated this action.')
  return (
    <BaseDialog
      open={open}
      onClose={close}
      title={tr('Replace existing Raspberry Pi Connect token?')}
    >
      <DialogHeading>{tr('Replace existing Raspberry Pi Connect token?')}</DialogHeading>
      <InfoText size={fontSize.sm} color={colors.formLabel}>
        {body}
      </InfoText>
      <DialogButtons>
        <ImButton
          onClick={replace}
          disabled={!allowAccept}
          accessibleDescription={tr('Replace the current token with the newly received one')}
        >
          {allowAccept ? tr('Replace token') : tr('Please wait…')}
        </ImButton>
        <ImButtonRed
          onClick={close}
          accessibleDescription={tr('Keep your current Raspberry Pi Connect token')}
        >
          {tr('Keep existing')}
        </ImButtonRed>
      </DialogButtons>
    </BaseDialog>
  )
}
