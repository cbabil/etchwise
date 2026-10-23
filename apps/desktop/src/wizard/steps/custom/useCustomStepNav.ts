/**
 * The navigation props every customisation step passes to `StepBase`:
 * Skip shown, upstream's Back / Skip accessible descriptions (translated
 * in the step's own context), Back and "Skip customisation" handlers.
 */
import { useTr } from '@/i18n'

import type { StepBaseProps } from '../../StepBase'
import { useWizard } from '../../WizardContext'

export type CustomStepNav = Pick<
  StepBaseProps,
  | 'showSkip'
  | 'nextAccessibleDescription'
  | 'backAccessibleDescription'
  | 'skipAccessibleDescription'
  | 'onBack'
  | 'onSkip'
>

export function useCustomStepNav(context: string, nextDescription: string): CustomStepNav {
  const tr = useTr(context)
  const wizard = useWizard()
  return {
    showSkip: true,
    nextAccessibleDescription: tr(nextDescription),
    backAccessibleDescription: tr('Return to previous step'),
    skipAccessibleDescription: tr(
      'Skip all customisation and proceed directly to writing the image',
    ),
    onBack: wizard.back,
    onSkip: () => wizard.skipCustomisation(),
  }
}
