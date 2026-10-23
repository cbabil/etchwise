import { act, fireEvent, screen } from '@testing-library/react'

import sdIcon from '@/assets/icons/ic_sd_storage_40px.svg'
import usbIcon from '@/assets/icons/ic_usb_40px.svg'
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

describe('StorageSelectionStep', () => {
  it('lists drives with icons, sizes and mount points, hiding system drives', async () => {
    await show()
    expect(screen.getByRole('heading', { name: 'Select your storage device' })).toBeVisible()
    expect(screen.getAllByRole('option')).toHaveLength(3)
    expect(option(/Apple SDXC Reader Media/)).toHaveTextContent('29.7 GB')
    expect(option(/Apple SDXC/)).toHaveTextContent('Mounted as /Volumes/bootfs')
    expect(option(/SanDisk Ultra/)).toHaveTextContent('64 GB')
    const icons = document.querySelectorAll('[data-testid="storage-row"] img')
    expect([...icons].map((i) => i.getAttribute('src'))).toEqual([sdIcon, usbIcon, sdIcon])
    expect(screen.queryByText('APPLE SSD')).toBeNull()
    expect(next()).toBeDisabled()
    expect(screen.getByRole('listbox')).toHaveAccessibleName(/Storage device list\. 2 devices\./)
  })

  it('polls the drive list every second', async () => {
    await show()
    const before = backend.called('getDriveList').length
    drives = [sd]
    await flush(1000)
    expect(backend.called('getDriveList').length).toBe(before + 1)
    expect(screen.getAllByRole('option')).toHaveLength(1)
  })

  it('shows read-only devices dimmed and refuses to select them', async () => {
    await show()
    const row = option(/Locked Card/)
    expect(row).toHaveTextContent('Read-only')
    expect(row.querySelector('[data-testid="storage-row"]')).toHaveStyle({ opacity: '0.5' })
    fireEvent.click(row)
    expect(backend.called('setDst').filter(([d]) => d !== '')).toHaveLength(0)
    expect(next()).toBeDisabled()
  })

  it('selects a drive, enables Next and advances', async () => {
    const { wizard } = await show()
    fireEvent.click(option(/SanDisk Ultra/))
    expect(backend.called('setDst').at(-1)).toEqual(['/dev/disk5', 64 * 1024 ** 3])
    expect(wizard().selectedStorageName).toBe('SanDisk Ultra')
    expect(option(/SanDisk Ultra/)).toHaveAttribute('aria-selected', 'true')
    fireEvent.click(next())
    expect(wizard().currentStep).toBe(Step.Hostname)
  })

  it('double-click and Enter select and advance', async () => {
    const { wizard } = await show()
    fireEvent.doubleClick(option(/Apple SDXC/))
    expect(wizard().currentStep).toBe(Step.Hostname)
    act(() => wizard().jumpTo(Step.Storage))
    const list = screen.getByRole('listbox')
    fireEvent.keyDown(list, { key: 'ArrowDown' })
    fireEvent.keyDown(list, { key: 'ArrowDown' })
    fireEvent.keyDown(list, { key: 'Enter' })
    await flush()
    expect(wizard().selectedStorageName).toBe('SanDisk Ultra')
    expect(wizard().currentStep).toBe(Step.Hostname)
  })

  it('asks before showing system drives and re-checks the filter on Keep filter on', async () => {
    await show()
    const filter = screen.getByRole('checkbox', { name: 'Exclude system drives' })
    expect(filter).toBeChecked()
    fireEvent.click(filter)
    const dialog = screen.getByRole('dialog')
    expect(dialog).toHaveTextContent('system drives will be shown')
    expect(dialog.querySelector('b')).toHaveTextContent('system drives will be shown')
    fireEvent.click(screen.getByRole('button', { name: /^KEEP FILTER ON/ }))
    expect(filter).toBeChecked()
    expect(screen.queryByText('APPLE SSD')).toBeNull()
    fireEvent.click(filter)
    fireEvent.click(screen.getByRole('button', { name: /^SHOW SYSTEM DRIVES/ }))
    await flush(500)
    expect(filter).not.toBeChecked()
    expect(option(/APPLE SSD/)).toBeVisible()
  })

  it('skips the unfilter warning when warnings are disabled', async () => {
    const { wizard } = await show()
    act(() => wizard().update({ disableWarnings: true }))
    fireEvent.click(screen.getByRole('checkbox', { name: 'Exclude system drives' }))
    expect(screen.queryByRole('dialog')).toBeNull()
    expect(option(/APPLE SSD/)).toBeVisible()
  })

  it('requires the typed drive name for a system drive, blocking paste', async () => {
    const { wizard } = await show()
    act(() => wizard().update({ disableWarnings: true }))
    fireEvent.click(screen.getByRole('checkbox', { name: 'Exclude system drives' }))
    fireEvent.click(option(/APPLE SSD/))
    expect(screen.getByRole('dialog')).toHaveTextContent('Mounted as: /, /System/Volumes/Data')
    const input = screen.getByRole('textbox', {
      name: /Drive name input\. Type exactly: APPLE SSD/,
    })
    const proceed = screen.getByRole('button', { name: /^CONTINUE/ })
    expect(proceed).toBeDisabled()
    expect(fireEvent.paste(input)).toBe(false)
    expect(fireEvent.keyDown(input, { key: 'v', metaKey: true })).toBe(false)
    expect(fireEvent.contextMenu(input)).toBe(false)
    fireEvent.change(input, { target: { value: 'APPLE SS' } })
    fireEvent.keyDown(input, { key: 'Enter' })
    expect(wizard().currentStep).toBe(Step.Storage)
    fireEvent.change(input, { target: { value: 'APPLE SSD' } })
    fireEvent.click(proceed)
    expect(backend.called('setDst').at(-1)?.[0]).toBe('/dev/disk0')
    expect(wizard().currentStep).toBe(Step.Hostname)
    expect(wizard().selectedStorageName).toBe('APPLE SSD')
  })

  it('cancels the system drive confirmation', async () => {
    const { wizard } = await show()
    act(() => wizard().update({ disableWarnings: true }))
    fireEvent.click(screen.getByRole('checkbox', { name: 'Exclude system drives' }))
    fireEvent.click(option(/APPLE SSD/))
    fireEvent.click(screen.getByRole('button', { name: /^CANCEL,/ }))
    await flush(500)
    expect(wizard().selectedStorageName).toBe('')
    expect(next()).toBeDisabled()
  })
})
