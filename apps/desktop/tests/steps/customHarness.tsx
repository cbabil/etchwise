/**
 * Test harness for the hostname / locale / user / Wi-Fi / SSH steps: each
 * test file mocks `@/api/imager` inline (`call` and `onImagerEvent` as
 * `vi.fn`s; a factory importing this file would deadlock), sets the backend answers with `answer({...})`, and renders a step once
 * the wizard has loaded (like `WizardContainer`, which waits for `ready`).
 */
import { act, render, screen, waitFor } from '@testing-library/react'
import type { ComponentType, ReactNode } from 'react'
import type { Mock } from 'vitest'

import { call } from '@/api/imager'
import type { WizardState } from '@/wizard/state'
import { useWizard, type WizardApi, WizardProvider } from '@/wizard/WizardContext'

import { Capture } from '../wizard/capture'

type Answers = Record<string, (...args: unknown[]) => unknown>

export const callMock = (): Mock => call as unknown as Mock

/** Backend answers by method name; everything else resolves to null. */
export function answer(answers: Answers): void {
  callMock().mockImplementation(async (method: string, ...args: unknown[]) =>
    answers[method] ? answers[method](...args) : null,
  )
}

/** Arguments of every call to `method`. */
export const callsTo = (method: string): unknown[][] =>
  callMock()
    .mock.calls.filter(([m]) => m === method)
    .map(([, ...args]) => args)

function Gate({ children }: { children: ReactNode }) {
  return useWizard().ready ? <>{children}</> : null
}

export interface Rendered {
  wizard: () => WizardApi
}

export async function renderStep(
  Step: ComponentType,
  initial: Partial<WizardState> = {},
): Promise<Rendered> {
  let current: WizardApi | null = null
  render(
    <WizardProvider showLanguageSelection={false} initial={initial}>
      <Capture
        onWizard={(w) => {
          current = w
        }}
      />
      <Gate>
        <Step />
      </Gate>
    </WizardProvider>,
  )
  await waitFor(() => expect(current?.ready).toBe(true))
  // Start-up picks the first step itself; move to the requested one afterwards.
  const step = initial.currentStep
  if (step !== undefined) act(() => (current as unknown as WizardApi).jumpTo(step))
  await screen.findByRole('heading', { level: 1, hidden: true })
  return { wizard: () => current as unknown as WizardApi }
}

export const nextButton = () => screen.getByRole('button', { name: /^Next/ })
