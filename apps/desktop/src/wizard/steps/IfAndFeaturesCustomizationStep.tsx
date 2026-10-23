/**
 * Upstream `IfAndFeaturesCustomizationStep.qml`: I2C / SPI / 1-Wire
 * switches, the Serial mode combo and USB gadget mode, each shown only
 * when the device and OS support it. Skips itself when nothing is
 * supported; enabling USB gadget mode asks for confirmation unless
 * warnings are disabled.
 */
import { Box } from '@mui/material'

import { useTr } from '@/i18n'
import { sizes } from '@/theme/tokens'

import { StepBase } from '../StepBase'
import { useWizard } from '../WizardContext'
import { FeaturesSection, InterfacesSection } from './custom2/IfFeaturesSections'
import { UsbGadgetDialog } from './custom2/UsbGadgetDialog'
import { useIfFeatures } from './custom2/useIfFeatures'

export const SKIP_IF_FEATURES = {
  piConnectEnabled: false,
  ifI2cEnabled: false,
  ifSpiEnabled: false,
  ifSerial: 'Disabled',
  featUsbGadgetEnabled: false,
} as const

export function IfAndFeaturesCustomizationStep() {
  const tr = useTr('IfAndFeaturesCustomizationStep')
  const wizard = useWizard()
  const features = useIfFeatures()
  const { caps, values } = features
  return (
    <StepBase
      title={tr('Customisation: Interfaces & Features')}
      subtitle={tr('Enable hardware interfaces and connectivity options.')}
      showSkip
      nextAccessibleDescription={tr(
        'Save interface and feature settings and continue to writing step',
      )}
      backAccessibleDescription={tr('Return to previous step')}
      skipAccessibleDescription={tr(
        'Skip all customisation and proceed directly to writing the image',
      )}
      onNext={features.next}
      onBack={wizard.back}
      onSkip={() => wizard.skipCustomisation(SKIP_IF_FEATURES)}
    >
      {caps && values ? (
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            gap: `${sizes.stepContentSpacing}px`,
            pr: '20px',
          }}
        >
          <InterfacesSection caps={caps} values={values} onChange={features.change} />
          <FeaturesSection caps={caps} values={values} onChange={features.change} />
        </Box>
      ) : null}
      <UsbGadgetDialog
        open={features.confirmOpen}
        onCancel={features.cancel}
        onAccept={features.accept}
      />
    </StepBase>
  )
}
