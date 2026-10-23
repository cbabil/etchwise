import { act, fireEvent, screen } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { setDst, setRemovedNotice } from '@/wizard/steps/storage/selectedDrive'
import { StorageSelectionStep } from '@/wizard/steps/StorageSelectionStep'

import { drive, flush, renderStep } from './writeFlowHarness'
import { backend } from './writeFlowImager'

vi.mock('@/api/imager', () => import('./writeFlowImager').then((m) => m.imagerMock))

const sd = drive()
const usb = drive({
  key: 'usb',
  device: '/dev/disk5',
  description: 'SanDisk Ultra',
  size: 64 * 1024 ** 3,
  isUsb: true,
  mountpoints: [],
})
const ro = drive({ key: 'ro', device: '/dev/disk6', description: 'Locked Card', isReadOnly: true })
const sys = drive({
  key: 'sys',
  device: '/dev/disk0',
  description: 'APPLE SSD',
  isSystem: true,
  isScsi: true,
  mountpoints: ['/', '/System/Volumes/Data'],
})

let drives: unknown[] = []
let error = ''

beforeEach(() => {
  vi.useFakeTimers()
  backend.reset()
  setDst('', 0)
  setRemovedNotice(false)
  drives = [sd, usb, ro, sys]
  error = ''
  backend.answers.getDriveList = () => ({ drives, error })
})
afterEach(() => vi.useRealTimers())

const show = () => renderStep(<StorageSelectionStep />, Step.Storage)
const next = () => screen.getByRole('button', { name: 'Next' })
const option = (name: RegExp) => screen.getByRole('option', { name })

describe('StorageSelectionStep errors and removal', () => {
  it('shows the enumeration error banner and status', async () => {
    drives = []
    error = 'permission denied'
    await show()
    expect(screen.getByRole('alert')).toHaveTextContent(
      'Could not list storage devices: permission denied',
    )
    expect(screen.getByRole('listbox')).toHaveAccessibleName(/This may be a permissions issue/)
  })

  it.each([
    [[], 'No storage devices found'],
    [[ro], 'All visible devices are read-only.'],
    [[sys], 'All devices are hidden by the filter.'],
  ])('announces why nothing can be selected', async (list, text) => {
    drives = list
    await show()
    expect(screen.getByRole('status')).toHaveTextContent(text)
  })

  it('clears the selection quietly when the selected device disappears', async () => {
    const { wizard } = await show()
    fireEvent.click(option(/SanDisk Ultra/))
    drives = [sd]
    await flush(1000)
    expect(wizard().selectedStorageName).toBe('')
    expect(backend.called('setDst').at(-1)).toEqual(['', 0])
    expect(next()).toBeDisabled()
    expect(screen.queryByRole('dialog')).toBeNull()
  })

  it('shows the storage removed dialog when sent back here', async () => {
    await show()
    act(() => setRemovedNotice(true))
    expect(screen.getByRole('dialog', { name: 'Storage device removed' })).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: /^OK/ }))
    await flush(500)
    expect(screen.queryByRole('dialog')).toBeNull()
  })
})
