/** The switches and option buttons of upstream `AppOptionsDialog.qml`. */
import { ImOptionPill } from '@/components/ImOptionPill'
import { useTr } from '@/i18n'

import { ImOptionButton } from '../common/ImOptionButton'
import type { AppOptionsEnv, AppOptionsValues } from './useAppOptions'

export const TELEMETRY_HELP_URL =
  'https://github.com/raspberrypi/rpi-imager?tab=readme-ov-file#anonymous-metrics-telemetry'

export interface AppOptionsListProps {
  values: AppOptionsValues
  env: AppOptionsEnv
  onChange: (patch: Partial<AppOptionsValues>) => void
  onDisableWarnings: (checked: boolean) => void
  onEditRepository: () => void
  onSelectKey: () => void
  onClearSettings: () => void
}

function Switches({ values, env, onChange, onDisableWarnings }: AppOptionsListProps) {
  const tr = useTr('AppOptionsDialog')
  return (
    <>
      <ImOptionPill
        label={tr('Play sound when finished')}
        accessibleDescription={
          env.beepAvailable
            ? tr('Play an audio notification when the image write process completes')
            : tr('Audio notification unavailable - no viable audio player found on this system')
        }
        checked={values.beep}
        disabled={!env.beepAvailable}
        onToggle={(beep) => onChange({ beep })}
      />
      <ImOptionPill
        label={tr('Eject media when finished')}
        accessibleDescription={tr(
          'Automatically eject the storage device when the write process completes successfully',
        )}
        checked={values.eject}
        onToggle={(eject) => onChange({ eject })}
      />
      <ImOptionPill
        label={tr('Enable anonymous statistics (telemetry)')}
        accessibleDescription={tr(
          'Send anonymous usage statistics to help improve Raspberry Pi Imager',
        )}
        {...(env.embedded ? {} : { helpLabel: tr('What is this?'), helpUrl: TELEMETRY_HELP_URL })}
        checked={values.telemetry}
        onToggle={(telemetry) => onChange({ telemetry })}
      />
      <ImOptionPill
        label={tr('Disable warnings')}
        accessibleDescription={tr(
          'Skip confirmation dialogs before writing images (advanced users only)',
        )}
        checked={values.disableWarnings}
        onToggle={onDisableWarnings}
      />
    </>
  )
}

function Buttons({
  values,
  env,
  onEditRepository,
  onSelectKey,
  onClearSettings,
}: AppOptionsListProps) {
  const tr = useTr('AppOptionsDialog')
  return (
    <>
      <ImOptionButton
        label={tr('Content Repository')}
        buttonText={tr('Edit')}
        accessibleDescription={tr(
          'Change the source of operating system images between official Raspberry Pi repository and custom sources',
        )}
        disabled={!env.idle}
        onClick={onEditRepository}
      />
      {env.secureBootVisible ? (
        <ImOptionButton
          label={tr('Secure Boot RSA Key')}
          buttonText={values.rsaKeyPath ? tr('Change') : tr('Select')}
          accessibleDescription={tr(
            'Select an RSA 2048-bit private key for signing boot images in secure boot mode',
          )}
          disabled={!env.idle}
          onClick={onSelectKey}
        />
      ) : null}
      <ImOptionButton
        label={tr('Saved Customisation')}
        buttonText={tr('Clear')}
        accessibleDescription={tr(
          'Remove all saved OS customisation settings such as hostname, WiFi, and user credentials',
        )}
        onClick={onClearSettings}
      />
    </>
  )
}

export function AppOptionsList(props: AppOptionsListProps) {
  return (
    <>
      <Switches {...props} />
      <Buttons {...props} />
    </>
  )
}
