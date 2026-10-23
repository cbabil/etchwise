/** Rendering helpers for the Language / Device / OS step tests. */
import { render } from '@testing-library/react'
import type { ReactElement } from 'react'

import type { WizardState } from '@/wizard/state'
import type { WizardApi } from '@/wizard/WizardContext'
import { WizardProvider } from '@/wizard/WizardContext'

import { Capture } from '../wizard/capture'

export interface Rendered {
  wizard: () => WizardApi
}

export function renderStep(step: ReactElement, initial: Partial<WizardState> = {}): Rendered {
  let latest: WizardApi | null = null
  render(
    <WizardProvider showLanguageSelection={false} initial={initial}>
      <Capture
        onWizard={(w) => {
          latest = w
        }}
      />
      {step}
    </WizardProvider>,
  )
  return {
    wizard: () => {
      if (!latest) throw new Error('wizard not rendered')
      return latest
    },
  }
}

/** Arguments of every `call(method, …)` so far. */
export async function callsOf(method: string): Promise<unknown[][]> {
  const { call } = await import('@/api/imager')
  return vi
    .mocked(call)
    .mock.calls.filter(([m]) => m === method)
    .map(([, ...args]) => args)
}
