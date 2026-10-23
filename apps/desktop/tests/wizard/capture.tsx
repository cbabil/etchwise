/** Test helper: hands the current `useWizard()` value to the test after every render. */
import { useEffect } from 'react'

import { useWizard, type WizardApi } from '@/wizard/WizardContext'

export function Capture({ onWizard }: { onWizard: (wizard: WizardApi) => void }) {
  const wizard = useWizard()
  useEffect(() => onWizard(wizard))
  return null
}
