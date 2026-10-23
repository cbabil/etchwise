import { act, fireEvent, render, screen } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { setDst, setRemovedNotice } from '@/wizard/steps/storage/selectedDrive'
import { StorageRemovedDialog } from '@/wizard/steps/storage/StorageRemovedDialog'
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

/** Opens the ERASE dialog, waits out the countdown and accepts. */
async function confirmAndStart() {
  fireEvent.click(nextButton(/^Write,/))
  await flush(2000)
  fireEvent.click(screen.getByRole('button', { name: /^I understand, erase and write/ }))
  snapshot('Preparing')
  await flush(300)
}

describe('WritingStep device removal', () => {
  it('returns to Storage with a notice when the device disappears before writing', async () => {
    render(<StorageRemovedDialog />)
    const { wizard } = await show()
    backend.answers.getDriveList = () => ({ drives: [], error: '' })
    await flush(1000)
    expect(wizard().currentStep).toBe(Step.Storage)
    expect(wizard().selectedStorageName).toBe('')
    expect(backend.called('setDst').at(-1)).toEqual(['', 0])
    expect(screen.getByRole('dialog', { name: 'Storage device removed' })).toBeVisible()
  })

  it('handles a write cancelled because the device was removed', async () => {
    const { wizard } = await show()
    await confirmAndStart()
    act(() => backend.emit('writeCancelledDueToDeviceRemoval'))
    expect(wizard().currentStep).toBe(Step.Storage)
  })
})
