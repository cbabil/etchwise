/**
 * Upstream `PiConnectCustomizationStep.qml`: enable Raspberry Pi Connect,
 * sign in through the browser (or paste a token once the 25 s countdown
 * ends) and validate hand-typed tokens on Next.
 */
import { Box } from '@mui/material'

import { ImButton } from '@/components/ImButton'
import { ImOptionPill } from '@/components/ImOptionPill'
import { ImTextField } from '@/components/ImTextField'
import { useTr } from '@/i18n'
import { spacing } from '@/theme/tokens'

import { StepBase } from '../StepBase'
import { useWizard } from '../WizardContext'
import { InvalidTokenDialog } from './custom2/InvalidTokenDialog'
import { CenteredContent, FormLabel, SectionContainer } from './custom2/layout'
import { canProceed, type ConnectState } from './custom2/piConnectState'
import { usePiConnect } from './custom2/usePiConnect'

export const CONNECT_HELP_URL = 'https://www.raspberrypi.com/software/connect/'

type Connect = ReturnType<typeof usePiConnect>

function placeholderFor(state: ConnectState, tr: (s: string, ...a: unknown[]) => string): string {
  if (state.received) return tr('Token received from browser')
  if (state.countdown > 0) return tr('Waiting for token (%1s)', state.countdown)
  return tr('Paste token here')
}

function TokenField({ connect }: { connect: Connect }) {
  const tr = useTr('PiConnectCustomizationStep')
  const { state } = connect
  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.xSmall}px` }}>
      <FormLabel id="connect-token-label">{tr('Authentication token:')}</FormLabel>
      <ImTextField
        fullWidth
        value={state.text}
        disabled={!state.fieldEnabled}
        placeholder={placeholderFor(state, tr)}
        onChange={(event) => connect.dispatch({ type: 'typed', text: event.target.value })}
        slotProps={{
          htmlInput: {
            'aria-labelledby': 'connect-token-label',
            'aria-description': tr(
              "Enter or paste the authentication token from Raspberry Pi Connect. The token will be automatically filled if you use the 'Open Raspberry Pi Connect' button to sign in.",
            ),
          },
        }}
      />
    </Box>
  )
}

function ConnectContent({ connect }: { connect: Connect }) {
  const tr = useTr('PiConnectCustomizationStep')
  const { state } = connect
  return (
    <CenteredContent>
      <SectionContainer>
        <ImOptionPill
          label={tr('Enable Raspberry Pi Connect')}
          accessibleDescription={tr(
            'Enable secure remote access to your Raspberry Pi through the Raspberry Pi Connect cloud service',
          )}
          helpLabel={tr('What is Raspberry Pi Connect?')}
          helpUrl={CONNECT_HELP_URL}
          checked={state.enabled}
          onToggle={connect.toggle}
        />
        {state.enabled && !state.received ? (
          <ImButton
            fullWidth
            onClick={connect.openConnect}
            accessibleDescription={tr(
              'Open the Raspberry Pi Connect website in your browser to sign in and receive an authentication token',
            )}
          >
            {tr('Open Raspberry Pi Connect')}
          </ImButton>
        ) : null}
        {state.enabled ? <TokenField connect={connect} /> : null}
      </SectionContainer>
    </CenteredContent>
  )
}

export function PiConnectCustomizationStep() {
  const tr = useTr('PiConnectCustomizationStep')
  const wizard = useWizard()
  const connect = usePiConnect()
  return (
    <StepBase
      title={tr('Customisation: Raspberry Pi Connect')}
      subtitle={tr('Sign in to receive a token and enable Raspberry Pi Connect')}
      showSkip
      nextEnabled={canProceed(connect.state)}
      nextAccessibleDescription={tr(
        'Save Raspberry Pi Connect settings and continue to next customisation step',
      )}
      backAccessibleDescription={tr('Return to previous step')}
      skipAccessibleDescription={tr(
        'Skip all customisation and proceed directly to writing the image',
      )}
      onNext={() => void connect.next()}
      onBack={wizard.back}
      onSkip={() => wizard.skipCustomisation({ piConnectEnabled: false })}
    >
      <ConnectContent connect={connect} />
      <InvalidTokenDialog
        open={connect.invalidOpen}
        onOk={connect.invalidOk}
        onDismiss={connect.dismissInvalid}
      />
    </StepBase>
  )
}
