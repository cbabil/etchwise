import { act, fireEvent, screen } from '@testing-library/react'

import { snapshotOf } from '@/wizard/state'
import { initialFlags } from '@/wizard/state'
import { Step } from '@/wizard/stepIds'
import { DoneStep } from '@/wizard/steps/DoneStep'

import { flush, renderStep } from './writeFlowHarness'
import { backend } from './writeFlowImager'

vi.mock('@/api/imager', () => import('./writeFlowImager').then((m) => m.imagerMock))

const close = vi.hoisted(() => vi.fn(async () => undefined))
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => ({ close }) }))

const completionSnapshot = {
  ...snapshotOf({ ...initialFlags, hostnameConfigured: true, wifiConfigured: true }),
  ifSerial: 'Console',
}

beforeEach(() => {
  vi.useFakeTimers()
  backend.reset()
  close.mockClear()
})
afterEach(() => {
  vi.useRealTimers()
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__
})

const show = () =>
  renderStep(<DoneStep />, Step.Done, {
    selectedDeviceName: 'Raspberry Pi 5',
    selectedOsName: 'Raspberry Pi OS (64-bit)',
    selectedStorageName: 'Apple SDXC Reader Media',
    completionSnapshot,
  })

describe('DoneStep', () => {
  it('lists the choices and applied customisations with check marks', async () => {
    await show()
    expect(screen.getByRole('heading', { name: 'Write complete!' })).toBeVisible()
    expect(screen.getByText('Your choices:')).toBeVisible()
    expect(screen.getByText('Apple SDXC Reader Media')).toBeVisible()
    const list = screen.getByRole('list', { name: /^3 customizations: / })
    expect(list).toHaveTextContent('Wi‑Fi configured')
    expect(list).toHaveTextContent('Serial configured')
    expect(list.querySelectorAll('svg[data-testid="CheckIcon"]')).toHaveLength(3)
    expect(
      screen.getByText('Please eject the storage device before removing it from your computer.'),
    )
    expect(screen.queryByRole('button', { name: /^Next|^Back/ })).toBeNull()
  })

  it('tells the user the device was ejected automatically', async () => {
    backend.answers.getBoolSetting = (key) => key === 'eject'
    await show()
    expect(
      screen.getByText(
        'The storage device was ejected automatically. You can now remove it safely.',
      ),
    ).toBeVisible()
  })

  it('hides the customisation list when nothing was applied', async () => {
    const { wizard } = await show()
    act(() => wizard().update({}))
    await renderStep(<DoneStep />, Step.Done)
    expect(screen.getAllByText('Your choices:')).toHaveLength(2)
    expect(screen.getAllByText('Customisations applied:')).toHaveLength(1)
  })

  it('Write Another returns to Storage in write-another mode', async () => {
    const { wizard } = await show()
    fireEvent.click(screen.getByRole('button', { name: /^Write Another/ }))
    expect(wizard().currentStep).toBe(Step.Storage)
    expect(wizard().writeAnotherMode).toBe(true)
    expect(wizard().selectedStorageName).toBe('')
    expect(backend.called('setDst').at(-1)).toEqual(['', 0])
  })

  it('Finish closes the window inside Tauri only', async () => {
    await show()
    fireEvent.click(screen.getByRole('button', { name: /^Finish/ }))
    await flush()
    expect(close).not.toHaveBeenCalled()
    ;(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {}
    fireEvent.click(screen.getByRole('button', { name: /^Finish/ }))
    await flush()
    expect(close).toHaveBeenCalledTimes(1)
  })

  it('Reboot replaces Finish in embedded mode', async () => {
    backend.answers.isEmbeddedMode = () => true
    await show()
    fireEvent.click(screen.getByRole('button', { name: /^Reboot,/ }))
    await flush()
    expect(backend.called('reboot')).toHaveLength(1)
    expect(close).not.toHaveBeenCalled()
  })
})
