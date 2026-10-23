/**
 * Upstream `RemoteAccessStep.qml`: an "Enable SSH" option pill (with a
 * help link outside embedded mode), then, enabled only while SSH is on,
 * the password / public-key radios and the SSH key manager.
 */
import { Box } from '@mui/material'

import { ImOptionPill } from '@/components/ImOptionPill'
import { ImRadioButton } from '@/components/ImRadioButton'
import { useTr } from '@/i18n'
import { spacing } from '@/theme/tokens'

import { StepBase } from '../StepBase'
import { FormLabel, SectionContainer, StepContent } from './custom/layout'
import { SshKeyManager } from './custom/SshKeyManager'
import { useCustomStepNav } from './custom/useCustomStepNav'
import { useRemoteAccess } from './custom/useRemoteAccess'

const CONTEXT = 'RemoteAccessStep'
const SSH_HELP_URL = 'https://www.raspberrypi.com/documentation/computers/remote-access.html#ssh'

type Ssh = ReturnType<typeof useRemoteAccess>

function AuthMechanism({ ssh }: { ssh: Ssh }) {
  const tr = useTr(CONTEXT)
  const off = !ssh.fields.enabled
  return (
    <Box sx={{ display: 'flex', gap: `${spacing.medium}px`, alignItems: 'flex-start' }}>
      <Box sx={{ pt: `${spacing.tiny + spacing.xxSmall}px` }}>
        <FormLabel
          isDisabled={off}
          description={tr(
            'Choose how you will authenticate when connecting to your Raspberry Pi via SSH. Password authentication uses the account credentials you configured. Public key authentication uses a cryptographic key pair and is more secure.',
          )}
        >
          {tr('Authentication mechanism:')}
        </FormLabel>
      </Box>
      <Box
        role="radiogroup"
        sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.xxSmall}px` }}
      >
        <ImRadioButton
          label={tr('Use password authentication')}
          name="ssh-auth"
          checked={!ssh.fields.publicKey}
          disabled={off}
          accessibleDescription={tr(
            'Allow SSH login using the username and password you configured in the previous step.',
          )}
          onSelect={() => ssh.set({ publicKey: false })}
        />
        <ImRadioButton
          label={tr('Use public key authentication')}
          name="ssh-auth"
          checked={ssh.fields.publicKey}
          disabled={off}
          accessibleDescription={tr(
            'Allow SSH login using a cryptographic key pair instead of a password. More secure than password authentication.',
          )}
          onSelect={() => ssh.set({ publicKey: true })}
        />
      </Box>
    </Box>
  )
}

function SshFields({ ssh }: { ssh: Ssh }) {
  const tr = useTr(CONTEXT)
  return (
    <SectionContainer>
      <ImOptionPill
        label={tr('Enable SSH')}
        checked={ssh.fields.enabled}
        onToggle={(enabled) => ssh.set({ enabled })}
        accessibleDescription={tr(
          'Enable secure shell access for remote command-line control of your Raspberry Pi',
        )}
        {...(ssh.embedded ? {} : { helpLabel: tr('Learn about SSH'), helpUrl: SSH_HELP_URL })}
      />
      <Box sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.smallPlus}px` }}>
        <AuthMechanism ssh={ssh} />
        {ssh.fields.publicKey ? (
          <SshKeyManager
            keys={ssh.fields.keys}
            disabled={!ssh.fields.enabled}
            onChange={(keys) => ssh.set({ keys })}
          />
        ) : null}
      </Box>
    </SectionContainer>
  )
}

export function RemoteAccessStep() {
  const tr = useTr(CONTEXT)
  const ssh = useRemoteAccess()
  const nav = useCustomStepNav(CONTEXT, 'Save SSH settings and continue to next customisation step')
  return (
    <StepBase
      title={tr('Customisation: SSH authentication')}
      subtitle={tr('Configure SSH access')}
      {...nav}
      nextEnabled={ssh.nextEnabled}
      onNext={() => void ssh.submit()}
    >
      <StepContent gap={spacing.small}>
        <SshFields ssh={ssh} />
      </StepContent>
    </StepBase>
  )
}
