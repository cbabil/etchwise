import { act, fireEvent, screen } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { setDst, setRemovedNotice } from '@/wizard/steps/storage/selectedDrive'
import { WritingStep } from '@/wizard/steps/WritingStep'

import { drive, flush, renderStep } from './writeFlowHarness'
import { backend } from './writeFlowImager'

vi.mock('@/api/imager', () => import('./writeFlowImager').then((m) => m.imagerMock))

let progress: Record<string, unknown> = {}
const snapshot = (state: string, extra: Record<string, unknown> = {}) => {
  progress = { state, bytesWritten: 0, writeTotal: 0, verifyNow: 0, verifyTotal: 0, ...extra }
}

beforeEach(() => {
  vi.useFakeTimers()
  backend.reset()
  setRemovedNotice(false)
  setDst('/dev/disk4', 32e9)
  snapshot('Idle')
  backend.answers.progress = () => progress
  backend.answers.readyToWrite = () => true
  backend.answers.isExtractSizeKnown = () => true
  backend.answers.getDriveList = () => ({ drives: [drive()], error: '' })
})
afterEach(() => vi.useRealTimers())

const initial = {
  selectedDeviceName: 'Raspberry Pi 5',
  selectedOsName: 'Raspberry Pi OS (64-bit)',
  selectedStorageName: 'Apple SDXC Reader Media',
}
const show = () => renderStep(<WritingStep />, Step.Writing, initial)
const nextButton = (name: RegExp) => screen.getByRole('button', { name })
const back = () => screen.getByRole('button', { name: /^Back/ })

/** Opens the ERASE dialog, waits out the countdown and accepts. */
async function confirmAndStart() {
  fireEvent.click(nextButton(/^Write,/))
  await flush(2000)
  fireEvent.click(screen.getByRole('button', { name: /^I understand, erase and write/ }))
  snapshot('Preparing')
  await flush(300)
}

describe('WritingStep summary', () => {
  it('shows the choices and the customisations to apply', async () => {
    const { wizard } = await show()
    act(() => wizard().update({ hostnameConfigured: true, sshEnabled: true, ifI2cEnabled: true }))
    expect(screen.getByRole('heading', { name: 'Write image' })).toBeVisible()
    expect(screen.getByText('Review your choices and write the image to the storage device'))
    expect(screen.getByText('Raspberry Pi 5')).toBeVisible()
    expect(screen.getByText('Raspberry Pi OS (64-bit)')).toBeVisible()
    expect(screen.getByText('Apple SDXC Reader Media')).toBeVisible()
    const list = screen.getByRole('list', { name: /^3 customizations: / })
    expect(list).toHaveTextContent('Hostname configured')
    expect(list).toHaveTextContent('I2C enabled')
  })

  it('hides customisations when none apply or customisation is unsupported', async () => {
    const { wizard } = await show()
    act(() => wizard().update({ ifSpiEnabled: true }))
    expect(screen.queryByText('Customisations to apply:')).toBeNull()
    act(() => wizard().update({ userConfigured: true, customizationSupported: false }))
    expect(screen.queryByText('Customisations to apply:')).toBeNull()
  })

  it('disables Write until the backend is ready', async () => {
    backend.answers.readyToWrite = () => false
    await show()
    expect(nextButton(/^Write,/)).toBeDisabled()
  })
})

describe('WritingStep write', () => {
  it('counts down before allowing the erase, and can be cancelled', async () => {
    await show()
    fireEvent.click(nextButton(/^Write,/))
    expect(screen.getByRole('dialog')).toHaveTextContent(
      'You are about to ERASE all data on: Apple SDXC Reader Media',
    )
    expect(screen.getByText('Please wait... 2')).toBeVisible()
    await flush(1000)
    expect(screen.getByText('Please wait... 1')).toBeVisible()
    await flush(1000)
    fireEvent.click(screen.getByRole('button', { name: /^Cancel,/ }))
    await flush(500)
    expect(screen.queryByRole('dialog')).toBeNull()
    expect(backend.called('startWrite')).toHaveLength(0)
  })

  it('writes, verifies and advances to Done', async () => {
    const { wizard } = await show()
    await confirmAndStart()
    expect(backend.called('startWrite')).toHaveLength(1)
    expect(wizard().isWriting).toBe(true)
    expect(back()).toBeDisabled()
    expect(screen.getByRole('status')).toHaveTextContent('Starting write process...')
    snapshot('Writing', { bytesWritten: 50, writeTotal: 100 })
    await flush(100)
    expect(screen.getByText('Writing... 50%')).toBeVisible()
    expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '50')
    expect(nextButton(/^Cancel write/)).toBeEnabled()
    snapshot('Verifying', { verifyNow: 25, verifyTotal: 100 })
    await flush(100)
    expect(screen.getByText('Verifying... 25%')).toBeVisible()
    expect(screen.getByText(/^Verifying written data/)).toBeVisible()
    fireEvent.click(nextButton(/^Skip verification/))
    expect(backend.called('skipCurrentVerification')).toHaveLength(1)
    snapshot('Finalizing')
    await flush(100)
    expect(screen.getByText('Finalising...')).toBeVisible()
    act(() => backend.emit('success'))
    expect(wizard().currentStep).toBe(Step.Done)
    expect(wizard().isWriting).toBe(false)
  })

  it('cancels a write and returns to the summary', async () => {
    const { wizard } = await show()
    await confirmAndStart()
    snapshot('Writing', { bytesWritten: 10, writeTotal: 100 })
    await flush(100)
    fireEvent.click(nextButton(/^Cancel write/))
    expect(backend.called('cancelWrite')).toHaveLength(1)
    expect(screen.getByText('Finalising…')).toBeVisible()
    act(() => backend.emit('cancelled'))
    expect(wizard().isWriting).toBe(false)
    expect(nextButton(/^Write,/)).toBeEnabled()
    expect(screen.getByText('Summary')).toBeVisible()
  })

  it('starts directly when warnings are disabled and shows indeterminate progress', async () => {
    backend.answers.isExtractSizeKnown = () => false
    const { wizard } = await show()
    act(() => wizard().update({ disableWarnings: true }))
    fireEvent.click(nextButton(/^Write,/))
    expect(screen.queryByRole('dialog')).toBeNull()
    expect(nextButton(/^Write,/)).toBeDisabled()
    await flush(300)
    snapshot('Writing', { bytesWritten: 5 * 1024 * 1024 })
    await flush(100)
    expect(screen.getByText('Writing... 5 MB written')).toBeVisible()
    expect(screen.getByRole('progressbar')).not.toHaveAttribute('aria-valuenow')
  })

  it('keeps Preparing while the backend still reports Idle', async () => {
    await show()
    await confirmAndStart()
    snapshot('Idle')
    await flush(200)
    expect(screen.getByText('Starting write process...')).toBeVisible()
  })

  it('shows the speed line and operation warnings from backend signals', async () => {
    await show()
    await confirmAndStart()
    snapshot('Writing', { bytesWritten: 1, writeTotal: 100 })
    await flush(100)
    act(() => {
      backend.emit('bottleneckStatusChanged', {
        status: 'Limited by storage device speed',
        throughputKBps: 20480,
      })
      backend.emit('operationWarning', 'Falling back to sync writes')
      backend.emit('preparationStatusUpdate', 'ignored while writing')
    })
    expect(screen.getByText('Limited by storage device speed (20 MB/s)')).toBeVisible()
    expect(screen.getByText(/Falling back to sync writes/)).toBeVisible()
  })

  it('returns to the summary on a backend error', async () => {
    const { wizard } = await show()
    await confirmAndStart()
    act(() => backend.emit('error', 'disk full'))
    expect(wizard().isWriting).toBe(false)
    expect(screen.getByText('Summary')).toBeVisible()
  })

  it('reports a rejected startWrite', async () => {
    backend.answers.startWrite = () => Promise.reject(new Error('No device'))
    await show()
    await confirmAndStart()
    expect(screen.getByRole('dialog', { name: 'Error' })).toHaveTextContent('No device')
    fireEvent.click(screen.getByRole('button', { name: /^Continue/ }))
    await flush(500)
    expect(screen.queryByRole('dialog')).toBeNull()
  })

  it('resumes polling a write already running on mount', async () => {
    snapshot('Writing', { bytesWritten: 30, writeTotal: 100 })
    const { wizard } = await show()
    await flush(100)
    expect(wizard().isWriting).toBe(true)
    expect(screen.getByText('Writing... 30%')).toBeVisible()
  })
})
