/**
 * Upstream `SecureBootCustomizationStep.qml`: pick the RSA private key
 * (saved as `secureboot_rsa_key`), then enable boot-partition signing,
 * which stays disabled until a key is selected. Shows the public key
 * fingerprint while enabled.
 */
import { Box } from '@mui/material'

import { ImOptionPill } from '@/components/ImOptionPill'
import { useTr } from '@/i18n'
import { colors, fontSize, spacing } from '@/theme/tokens'

import { InfoText } from '../dialogs/common/dialogParts'
import { ImOptionButton } from '../dialogs/common/ImOptionButton'
import { StepBase } from '../StepBase'
import { useWizard } from '../WizardContext'
import { CenteredContent, SectionContainer } from './custom2/layout'
import { OtpSection } from './custom2/OtpSection'
import { useSecureBoot } from './custom2/useSecureBoot'

export const SECURE_BOOT_HELP_URL =
  'https://github.com/raspberrypi/usbboot/blob/master/secure-boot-recovery/README.md'

function SigningInfo({ fingerprint }: { fingerprint: string }) {
  const tr = useTr('SecureBootCustomizationStep')
  const shown = fingerprint || tr('(unable to compute)')
  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.medium}px` }}>
      <InfoText>
        {tr('Your boot partition will be signed using the selected RSA private key.')}
      </InfoText>
      <InfoText>
        {tr('This will create boot.img and boot.sig files required for Raspberry Pi Secure Boot.')}
      </InfoText>
      <Box sx={{ mt: `${spacing.small - spacing.medium}px` }}>
        <InfoText>{tr('Public Key Fingerprint: %1', shown)}</InfoText>
      </Box>
    </Box>
  )
}

/** Qt `Text.ElideMiddle` for long key paths. */
export function elideMiddle(text: string, max = 64): string {
  if (text.length <= max) return text
  const half = Math.floor((max - 1) / 2)
  return `${text.slice(0, half)}\u2026${text.slice(text.length - half)}`
}

function SelectedKey({ keyPath }: { keyPath: string }) {
  const tr = useTr('SecureBootCustomizationStep')
  return (
    <Box
      title={keyPath}
      sx={{ fontSize: fontSize.xs, color: colors.textDescription, whiteSpace: 'nowrap' }}
    >
      {tr('Selected: %1', elideMiddle(keyPath))}
    </Box>
  )
}

function SigningPill({ secure }: { secure: ReturnType<typeof useSecureBoot> }) {
  const tr = useTr('SecureBootCustomizationStep')
  return (
    <Box sx={{ mt: `${spacing.medium}px` }}>
      <ImOptionPill
        label={tr('Enable Secure Boot Signing')}
        accessibleDescription={tr(
          'Sign the boot partition with your RSA key to enable secure boot verification on Raspberry Pi',
        )}
        helpLabel={tr('Learn about Secure Boot')}
        helpUrl={SECURE_BOOT_HELP_URL}
        checked={secure.checked}
        onToggle={secure.toggle}
        disabled={!secure.keyPath}
      />
    </Box>
  )
}

function SecureBootContent({ secure }: { secure: ReturnType<typeof useSecureBoot> }) {
  const tr = useTr('SecureBootCustomizationStep')
  return (
    <CenteredContent>
      <SectionContainer>
        <ImOptionButton
          label={tr('RSA Private Key')}
          buttonText={secure.keyPath ? tr('Change') : tr('Select')}
          accessibleDescription={tr(
            'Select an RSA 2048-bit private key for signing boot images in secure boot mode',
          )}
          onClick={() => void secure.selectKey()}
        />
        {secure.keyPath ? <SelectedKey keyPath={secure.keyPath} /> : null}
        <SigningPill secure={secure} />
        {secure.checked ? <SigningInfo fingerprint={secure.fingerprint} /> : null}
        {secure.keyPath ? null : (
          <InfoText color={colors.formLabelError}>
            {tr('Please select an RSA private key above to enable secure boot signing.')}
          </InfoText>
        )}
        {secure.rpiboot ? <OtpSection keyConfigured={Boolean(secure.keyPath)} /> : null}
      </SectionContainer>
    </CenteredContent>
  )
}

export function SecureBootCustomizationStep() {
  const tr = useTr('SecureBootCustomizationStep')
  const wizard = useWizard()
  const secure = useSecureBoot()
  return (
    <StepBase
      title={tr('Customisation: Secure Boot')}
      subtitle={tr('Configure secure boot image signing')}
      showSkip
      nextAccessibleDescription={tr(
        'Save secure boot settings and continue to next customisation step',
      )}
      backAccessibleDescription={tr('Return to previous step')}
      skipAccessibleDescription={tr(
        'Skip all customisation and proceed directly to writing the image',
      )}
      onNext={secure.next}
      onBack={wizard.back}
      onSkip={() => wizard.skipCustomisation({ secureBootEnabled: false })}
    >
      <SecureBootContent secure={secure} />
    </StepBase>
  )
}
