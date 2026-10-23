/** Harness rendering one step inside a ready `WizardProvider`, plus sample drives. */
import { act, render } from '@testing-library/react'
import { type ReactNode, useEffect } from 'react'

import type { WizardState } from '@/wizard/state'
import { useWizard, type WizardApi, WizardProvider } from '@/wizard/WizardContext'

/** Advances fake timers and flushes the promises they resolve. */
export const flush = (ms = 0) => act(() => vi.advanceTimersByTimeAsync(ms))

function Gate({
  step,
  children,
  onWizard,
}: {
  step: number
  children: ReactNode
  onWizard: (w: WizardApi) => void
}) {
  const wizard = useWizard()
  const { ready, jumpTo } = wizard
  useEffect(() => {
    onWizard(wizard)
  })
  useEffect(() => {
    if (ready) jumpTo(step)
  }, [ready, jumpTo, step])
  return ready ? <>{children}</> : null
}

/** Renders `ui` once the wizard is ready and at `step`; `wizard()` reads the live API. */
export async function renderStep(ui: ReactNode, step: number, initial: Partial<WizardState> = {}) {
  let api: WizardApi | null = null
  const result = render(
    <WizardProvider initial={initial} showLanguageSelection={false}>
      <Gate
        step={step}
        onWizard={(w) => {
          api = w
        }}
      >
        {ui}
      </Gate>
    </WizardProvider>,
  )
  await flush()
  const wizard = (): WizardApi => {
    if (!api) throw new Error('wizard not ready')
    return api
  }
  return { ...result, wizard }
}

export const drive = (overrides: Record<string, unknown> = {}) => ({
  key: '/dev/disk4:32000000000',
  device: '/dev/disk4',
  description: 'Apple SDXC Reader Media',
  size: 31914983424,
  isUsb: false,
  isScsi: false,
  isReadOnly: false,
  isSystem: false,
  isRpiboot: false,
  mountpoints: ['/Volumes/bootfs'],
  childDevices: [],
  ...overrides,
})
