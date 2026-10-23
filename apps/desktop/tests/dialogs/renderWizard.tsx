/** Renders a step or dialog inside a ready `WizardProvider`, optionally on a given step. */
import { render, waitFor } from '@testing-library/react'
import { type ReactNode, useEffect, useState } from 'react'

import type { WizardState } from '@/wizard/state'
import { useWizard, type WizardApi, WizardProvider } from '@/wizard/WizardContext'

function Gate({ step, children }: { step: number | undefined; children: ReactNode }) {
  const wizard = useWizard()
  const [arrived, setArrived] = useState(step === undefined)
  if (!arrived && wizard.ready && wizard.currentStep === step) setArrived(true)
  const { ready, jumpTo } = wizard
  useEffect(() => {
    if (ready && step !== undefined) jumpTo(step)
  }, [ready, jumpTo, step])
  return ready && arrived ? children : null
}

export async function renderInWizard(
  ui: ReactNode,
  options: { step?: number; initial?: Partial<WizardState> } = {},
) {
  const ref: { current: WizardApi | null } = { current: null }
  function Capture() {
    const wizard = useWizard()
    useEffect(() => {
      ref.current = wizard
    })
    return null
  }
  const result = render(
    <WizardProvider
      showLanguageSelection={false}
      {...(options.initial ? { initial: options.initial } : {})}
    >
      <Capture />
      <Gate step={options.step}>{ui}</Gate>
    </WizardProvider>,
  )
  await waitFor(() => expect(ref.current?.ready).toBe(true))
  if (options.step !== undefined)
    await waitFor(() => expect(ref.current?.currentStep).toBe(options.step))
  const wizard = (): WizardApi => {
    if (!ref.current) throw new Error('wizard not ready')
    return ref.current
  }
  return { ...result, wizard }
}
