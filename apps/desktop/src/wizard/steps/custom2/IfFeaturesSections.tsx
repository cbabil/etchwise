/** The "Interfaces" and "Features" sections of upstream `IfAndFeaturesCustomizationStep.qml`. */
import { Box } from '@mui/material'

import { ImComboBox } from '@/components/ImComboBox'
import { ImOptionPill } from '@/components/ImOptionPill'
import { useTr } from '@/i18n'
import { colors, fontSize, spacing } from '@/theme/tokens'

import { type IfCaps, type IfValues, serialOptions } from './ifFeatures'
import { FormLabel, SectionContainer } from './layout'

export const USB_GADGET_HELP_URL =
  'https://github.com/raspberrypi/rpi-usb-gadget?tab=readme-ov-file'

export interface SectionProps {
  caps: IfCaps
  values: IfValues
  onChange: (patch: Partial<IfValues>) => void
}

function SectionTitle({ children }: { children: string }) {
  return (
    <Box
      component="h2"
      sx={{ m: 0, fontSize: fontSize.xl, fontWeight: 700, color: colors.formLabel }}
    >
      {children}
    </Box>
  )
}

function SerialRow({ caps, values, onChange }: SectionProps) {
  const tr = useTr('IfAndFeaturesCustomizationStep')
  const options = serialOptions(caps)
  const labels = options.map((o) => (o === 'Console' ? tr('Console') : o))
  return (
    <Box sx={{ display: 'flex', alignItems: 'center', gap: `${spacing.medium}px` }}>
      <Box sx={{ flex: 1 }}>
        <FormLabel bold>{tr('Enable Serial:')}</FormLabel>
      </Box>
      <Box sx={{ width: 200 }}>
        <ImComboBox
          options={labels}
          value={values.serialIndex}
          onChange={(serialIndex) => onChange({ serialIndex })}
          ariaLabel={`${tr('Enable Serial:')} ${labels[values.serialIndex] ?? ''}`}
        />
      </Box>
    </Box>
  )
}

export function InterfacesSection({ caps, values, onChange }: SectionProps) {
  const tr = useTr('IfAndFeaturesCustomizationStep')
  if (!(caps.i2c || caps.spi || caps.onewire || caps.serial)) return null
  const pill = (key: 'i2c' | 'spi' | 'onewire', label: string, description: string) =>
    caps[key] ? (
      <ImOptionPill
        label={label}
        accessibleDescription={description}
        checked={values[key]}
        onToggle={(checked) => onChange({ [key]: checked })}
      />
    ) : null
  return (
    <SectionContainer>
      <Box sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.small}px` }}>
        <SectionTitle>{tr('Interfaces')}</SectionTitle>
        {pill(
          'i2c',
          tr('Enable I2C'),
          tr(
            'Enable the I2C (Inter-Integrated Circuit) interface for connecting sensors and other low-speed peripherals',
          ),
        )}
        {pill(
          'spi',
          tr('Enable SPI'),
          tr(
            'Enable the SPI (Serial Peripheral Interface) for high-speed communication with displays and sensors',
          ),
        )}
        {pill(
          'onewire',
          tr('Enable 1-Wire'),
          tr(
            'Enable the 1-Wire interface for connecting temperature sensors and other Dallas/Maxim devices',
          ),
        )}
        {caps.serial ? <SerialRow caps={caps} values={values} onChange={onChange} /> : null}
      </Box>
    </SectionContainer>
  )
}

export function FeaturesSection({ caps, values, onChange }: SectionProps) {
  const tr = useTr('IfAndFeaturesCustomizationStep')
  if (!caps.usbOtg) return null
  return (
    <SectionContainer>
      <Box sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.small}px` }}>
        <SectionTitle>{tr('Features')}</SectionTitle>
        <ImOptionPill
          label={tr('Enable USB Gadget Mode')}
          accessibleDescription={tr(
            'Enable USB device mode to use your Raspberry Pi as a USB peripheral for networking and storage',
          )}
          helpLabel={tr('Learn more about USB Gadget Mode')}
          helpUrl={USB_GADGET_HELP_URL}
          checked={values.usbGadget}
          onToggle={(usbGadget) => onChange({ usbGadget })}
        />
      </Box>
    </SectionContainer>
  )
}
