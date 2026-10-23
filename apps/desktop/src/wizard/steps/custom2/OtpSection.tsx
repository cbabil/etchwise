/**
 * Upstream's "OTP Secure Boot Provisioning" box, shown only when an
 * rpiboot device is selected. The choice stays local: the wizard has no
 * `otpProvisioningEnabled` flag yet.
 */
import { Box } from '@mui/material'
import { useState } from 'react'

import { ImOptionPill } from '@/components/ImOptionPill'
import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

import { InfoText } from '../../dialogs/common/dialogParts'

export function OtpSection({ keyConfigured }: { keyConfigured: boolean }) {
  const tr = useTr('SecureBootCustomizationStep')
  const [checked, setChecked] = useState(false)
  return (
    <Box
      sx={{
        mt: `${spacing.large - sizes.stepContentSpacing}px`,
        p: `${spacing.medium}px`,
        border: `1px solid ${colors.titleSeparator}`,
        borderRadius: `${sizes.sectionBorderRadius}px`,
        display: 'flex',
        flexDirection: 'column',
        gap: `${spacing.xSmall}px`,
      }}
    >
      <InfoText size={fontSize.sm} bold color={colors.formLabel}>
        {tr('OTP Secure Boot Provisioning')}
      </InfoText>
      <InfoText>
        {tr(
          "Program the secure boot public key hash into the device's OTP memory. This is a permanent, irreversible operation.",
        )}
      </InfoText>
      <ImOptionPill
        label={tr('Provision Secure Boot OTP')}
        accessibleDescription={tr('Permanently program the secure boot key into device OTP memory')}
        checked={checked}
        onToggle={setChecked}
        disabled={!keyConfigured}
      />
      {checked ? (
        <InfoText color={colors.otpWarning}>
          {tr(
            'WARNING: After OTP provisioning, this device will ONLY boot images signed with the selected key. This cannot be undone.',
          )}
        </InfoText>
      ) : null}
    </Box>
  )
}
