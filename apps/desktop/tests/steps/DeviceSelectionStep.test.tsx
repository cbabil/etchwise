import { act, fireEvent, screen, waitFor } from '@testing-library/react'

import { mockEmit } from '@/api/mock'
import { Step } from '@/wizard/stepIds'
import { DeviceSelectionStep } from '@/wizard/steps/DeviceSelectionStep'
import { resolveIcon } from '@/wizard/steps/os/ListIcon'

import { answers, resetAnswers } from './selectionStepImager'
import { callsOf, renderStep } from './stepTestBackend'

vi.mock('@/api/imager', async () => (await import('./selectionStepImager')).fakeImager())

const devices = [
  { name: 'Raspberry Pi 5', description: 'Pi 5 boards', icon: 'icons/erase.png', tags: [] },
  { name: 'Raspberry Pi 4', description: 'Pi 4 boards', icon: 'https://x/pi4.png', tags: [] },
  { name: 'No filtering', description: 'Show everything', icon: '', isUsbBootConnected: true },
]

beforeEach(() => resetAnswers({ getHWList: () => ({ devices, currentIndex: -1 }) }))

const list = () => screen.findByRole('listbox', { name: /Device selection list\. 3 devices/ })
const next = () => screen.getByRole('button', { name: 'Next' })

describe('DeviceSelectionStep', () => {
  it('lists devices with icons, descriptions and the USB note; Next waits for a choice', async () => {
    renderStep(<DeviceSelectionStep />)
    await list()
    expect(screen.getByText('Pi 4 boards')).toBeInTheDocument()
    expect(screen.getByText('Connected via USB')).toBeInTheDocument()
    expect(document.querySelectorAll('img')).toHaveLength(2)
    expect(next()).toBeDisabled()
    expect(screen.queryByRole('button', { name: 'Back' })).toBeNull()
  })

  it('a click selects the device on the backend and names it in the wizard', async () => {
    const { wizard } = renderStep(<DeviceSelectionStep />)
    await list()
    fireEvent.click(screen.getByText('Raspberry Pi 4'))
    expect(await callsOf('setHWIndex')).toContainEqual([1])
    expect(wizard().selectedDeviceName).toBe('Raspberry Pi 4')
    expect(next()).toBeEnabled()
    fireEvent.click(next())
    expect(wizard().currentStep).toBe(Step.OS)
  })

  it('highlights the backend default device and enables Next', async () => {
    answers.getHWList = () => ({ devices, currentIndex: 0 })
    renderStep(<DeviceSelectionStep />)
    await waitFor(() => expect(next()).toBeEnabled())
    expect(screen.getAllByRole('option')[0]).toHaveAttribute('aria-selected', 'true')
  })

  it('Enter selects and advances; double-click does too', async () => {
    const { wizard } = renderStep(<DeviceSelectionStep />)
    const listbox = await list()
    fireEvent.keyDown(listbox, { key: 'ArrowDown' })
    fireEvent.keyDown(listbox, { key: 'ArrowDown' })
    fireEvent.keyDown(listbox, { key: 'Enter' })
    await waitFor(() => expect(wizard().currentStep).toBe(Step.OS))
    expect(wizard().selectedDeviceName).toBe('Raspberry Pi 4')
    act(() => wizard().jumpTo(Step.Device))
    fireEvent.doubleClick(screen.getByText('No filtering'))
    await waitFor(() => expect(wizard().currentStep).toBe(Step.OS))
    expect(wizard().selectedDeviceName).toBe('No filtering')
  })

  it('shows the offline placeholder with Retry, Next stays usable', async () => {
    resetAnswers({ getHWList: () => ({ devices: [], currentIndex: -1 }) })
    answers.isOsListUnavailable = () => true
    renderStep(<DeviceSelectionStep />)
    await screen.findByRole('heading', { name: 'Unable to load device list' })
    expect(next()).toBeEnabled()
    fireEvent.click(screen.getByRole('button', { name: /^Retry/ }))
    expect(await callsOf('beginOSListFetch')).toHaveLength(1)
    answers.isOsListUnavailable = () => false
    answers.getHWList = () => ({ devices, currentIndex: -1 })
    act(() => mockEmit({ name: 'osListUnavailableChanged', data: null }))
    await list()
  })

  it('reloads an empty list when osListPrepared arrives', async () => {
    answers.getHWList = () => ({ devices: [], currentIndex: -1 })
    renderStep(<DeviceSelectionStep />)
    await screen.findByRole('listbox', { name: /No devices/ })
    answers.getHWList = () => ({ devices: devices.slice(0, 1), currentIndex: -1 })
    act(() => mockEmit({ name: 'osListPrepared', data: null }))
    await screen.findByRole('listbox', { name: /1 device\./ })
  })

  it('reloads a filled list when osListPrepared arrives (extra repository merged)', async () => {
    answers.getHWList = () => ({ devices: devices.slice(0, 1), currentIndex: -1 })
    renderStep(<DeviceSelectionStep />)
    await screen.findByRole('listbox', { name: /1 device\./ })
    answers.getHWList = () => ({ devices: devices.slice(0, 2), currentIndex: -1 })
    act(() => mockEmit({ name: 'osListPrepared', data: null }))
    await screen.findByRole('listbox', { name: /2 devices\./ })
  })

  it('resolves bundled icons and keeps remote ones', () => {
    expect(resolveIcon('icons/erase.png')).toMatch(/erase/)
    expect(resolveIcon('../icons/use_custom.png')).toMatch(/use_custom/)
    expect(resolveIcon('icons/missing.png')).toBe('icons/missing.png')
    expect(resolveIcon('https://x/y.png')).toBe('https://x/y.png')
    expect(resolveIcon(undefined)).toBe('')
  })

  describe('Raspberry Pi | Other tabs', () => {
    const mixed = [
      devices[0],
      { name: 'Intel NUC', description: 'x86-64 PCs', icon: '', group: 'other' },
      devices[2],
    ]
    const tab = (name: string) => screen.getByRole('tab', { name })

    it('lists official devices under Raspberry Pi and extra ones under Other', async () => {
      answers.getHWList = () => ({ devices: mixed, currentIndex: -1 })
      renderStep(<DeviceSelectionStep />)
      await screen.findByRole('listbox', { name: /2 devices\./ })
      expect(tab('Raspberry Pi')).toHaveAttribute('aria-selected', 'true')
      expect(screen.getByText('No filtering')).toBeInTheDocument()
      expect(screen.queryByText('Intel NUC')).toBeNull()
      fireEvent.click(tab('Other'))
      await screen.findByRole('listbox', { name: /1 device\./ })
      expect(screen.getByText('Intel NUC')).toBeInTheDocument()
      expect(screen.queryByText('Raspberry Pi 5')).toBeNull()
    })

    it('selects with the device position in the full list', async () => {
      answers.getHWList = () => ({ devices: mixed, currentIndex: -1 })
      const { wizard } = renderStep(<DeviceSelectionStep />)
      await screen.findByRole('listbox', { name: /2 devices\./ })
      fireEvent.click(tab('Other'))
      fireEvent.click(await screen.findByText('Intel NUC'))
      expect(await callsOf('setHWIndex')).toContainEqual([1])
      expect(wizard().selectedDeviceName).toBe('Intel NUC')
      expect(next()).toBeEnabled()
    })

    it('opens on the tab of the selected device', async () => {
      answers.getHWList = () => ({ devices: mixed, currentIndex: 1 })
      renderStep(<DeviceSelectionStep />)
      await screen.findByRole('listbox', { name: /1 device\./ })
      expect(tab('Other')).toHaveAttribute('aria-selected', 'true')
      expect(screen.getByRole('option', { name: /Intel NUC/ })).toHaveAttribute(
        'aria-selected',
        'true',
      )
    })

    it('explains how to add devices when Other is empty', async () => {
      renderStep(<DeviceSelectionStep />)
      await list()
      fireEvent.click(tab('Other'))
      expect(
        await screen.findByText('Add devices with App Options > Extra Repository'),
      ).toBeInTheDocument()
    })
  })
})
