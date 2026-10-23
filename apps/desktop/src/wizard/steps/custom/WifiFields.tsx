/**
 * The form of upstream `WifiCustomizationStep.qml`: SECURE / OPEN toggle
 * tabs, then a two-column grid with SSID, password, confirmation, the
 * one-line red error text (blank keeps its height) and "Hidden SSID".
 */
import { Box } from '@mui/material'

import { ImCheckBox } from '@/components/ImCheckBox'
import { ImPasswordField } from '@/components/ImPasswordField'
import { ImTextField } from '@/components/ImTextField'
import { ImToggleTab } from '@/components/ImToggleTab'
import { useTr } from '@/i18n'
import { colors, fontSize, spacing } from '@/theme/tokens'

import { FormGrid, FormLabel, FormRow } from './layout'
import type { WifiForm } from './useWifiForm'

const CONTEXT = 'WifiCustomizationStep'

function ModeTabs({ form }: { form: WifiForm }) {
  const tr = useTr(CONTEXT)
  return (
    <FormRow gap={spacing.small}>
      <ImToggleTab
        active={form.fields.mode === 'secure'}
        accessibleDescription={tr(
          'Configure Wi-Fi for a password-protected network with WPA2/WPA3 encryption',
        )}
        onClick={() => form.setMode('secure')}
      >
        {tr('Secure network')}
      </ImToggleTab>
      <ImToggleTab
        active={form.fields.mode === 'open'}
        accessibleDescription={tr(
          'Configure Wi-Fi for an unencrypted network without password protection',
        )}
        onClick={() => form.setMode('open')}
      >
        {tr('Open network')}
      </ImToggleTab>
    </FormRow>
  )
}

function PasswordRows({ form }: { form: WifiForm }) {
  const tr = useTr(CONTEXT)
  const common = useTr('CommonStrings')
  const keep = form.canKeep
  const pwDescription = keep
    ? 'Enter a new Wi-Fi password, or leave blank to keep the previously saved password. Must be 8-63 characters or a 64-character hexadecimal key.'
    : 'Enter your Wi-Fi network password. Must be 8-63 characters or a 64-character hexadecimal key. You will need to re-enter it in the next field to confirm.'
  const confirmDescription = keep
    ? 'Re-enter the new Wi-Fi password to confirm, or leave blank to keep the previously saved password.'
    : 'Re-enter the Wi-Fi password to confirm it matches.'
  return (
    <>
      <FormLabel htmlFor="wifi-password" description={tr(pwDescription)}>
        {common('Password:')}
      </FormLabel>
      <ImPasswordField
        id="wifi-password"
        fullWidth
        placeholder={keep ? tr('Saved (hidden) — leave blank to keep') : tr('Network password')}
        value={form.fields.password}
        onChange={(password) => form.set({ password })}
      />
      <FormLabel htmlFor="wifi-password-confirm" description={tr(confirmDescription)}>
        {tr('Confirm password:')}
      </FormLabel>
      <ImPasswordField
        id="wifi-password-confirm"
        fullWidth
        placeholder={keep ? tr('Re-enter to change password') : tr('Re-enter password')}
        value={form.fields.confirm}
        onChange={(confirm) => form.set({ confirm })}
      />
      <Box />
      <Box
        role={form.error ? 'alert' : undefined}
        sx={{ fontSize: fontSize.xs, color: colors.formLabelError, minHeight: '1.2em' }}
      >
        {form.error ? tr(form.error) : ' '}
      </Box>
    </>
  )
}

export function WifiFields({ form }: { form: WifiForm }) {
  const tr = useTr(CONTEXT)
  return (
    <>
      <ModeTabs form={form} />
      <FormGrid>
        <FormLabel
          htmlFor="wifi-ssid"
          description={tr(
            'Enter the network name (SSID) of your Wi-Fi network. This is the name that appears when you search for available networks.',
          )}
        >
          {tr('SSID:')}
        </FormLabel>
        <ImTextField
          id="wifi-ssid"
          fullWidth
          placeholder={tr('Network name')}
          value={form.fields.ssid}
          onChange={(e) => form.set({ ssid: e.target.value })}
        />
        {form.showPw ? <PasswordRows form={form} /> : null}
        <ImCheckBox
          label={tr('Hidden SSID')}
          checked={form.fields.hidden}
          accessibleDescription={tr(
            'Check this if your Wi-Fi network does not broadcast its name and requires manual SSID entry to connect.',
          )}
          onChange={(hidden) => form.set({ hidden })}
        />
      </FormGrid>
    </>
  )
}
