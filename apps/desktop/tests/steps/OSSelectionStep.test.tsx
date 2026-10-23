import { act, fireEvent, screen, waitFor } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { resetUpdatePopupShown } from '@/wizard/steps/os/useListMetadata'
import { OSSelectionStep } from '@/wizard/steps/OSSelectionStep'

import { group, lite, mainList, piOs } from './osFixtures'
import { resetAnswers } from './selectionStepImager'
import { callsOf, renderStep } from './stepTestBackend'

vi.mock('@/api/imager', async () => (await import('./selectionStepImager')).fakeImager())

beforeEach(() => {
  resetUpdatePopupShown()
  resetAnswers({
    getOSList: () => mainList,
    getSublist: () => [lite],
    isCached: (hash) => hash === 'bb',
    checkSWCapability: (cap) => cap === 'rpi_connect',
    imageSupportsCustomization: () => true,
    imageSupportsCcRpi: () => true,
    checkHWAndSWCapability: (cap) => cap === 'spi',
    imagerMetadata: () => ({}),
  })
})

const mainListbox = () => screen.findByRole('listbox', { name: /^Operating system list\. 4/ })
const next = () => screen.getByRole('button', { name: 'Next' })

async function atOsStep() {
  const rendered = renderStep(<OSSelectionStep />)
  await mainListbox()
  act(() => rendered.wizard().jumpTo(Step.OS))
  return rendered
}

describe('OSSelectionStep rows', () => {
  it('shows names, recommended suffix, status lines and release dates', async () => {
    await atOsStep()
    expect(screen.getByText('A port of Debian (Recommended)')).toBeInTheDocument()
    expect(screen.getByText('Online - 1.3 GB download')).toBeInTheDocument()
    expect(screen.getByText('Released: 2026-10-06')).toBeInTheDocument()
    expect(screen.getByText('Format card as FAT32')).toBeInTheDocument()
    expect(screen.getByText('Use custom')).toBeInTheDocument()
    expect(next()).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Back' })).toBeEnabled()
  })
})

describe('OSSelectionStep selection', () => {
  it('a click sets the source and capability flags without advancing', async () => {
    const { wizard } = await atOsStep()
    act(() => wizard().updateCustomizationSettings({ enableI2C: true, secureBootEnabled: true }))
    fireEvent.click(screen.getByText(piOs.name))
    await waitFor(() => expect(next()).toBeEnabled())
    expect((await callsOf('setSrc')).at(-1)?.[0]).toMatchObject({
      url: piOs.url,
      downloadLen: piOs.image_download_size,
      extractLen: 6000,
      expectedHash: 'aa',
      parentCategory: '',
      osName: piOs.name,
      initFormat: 'cloudinit-rpi',
      releaseDate: '2026-10-06',
    })
    expect(await callsOf('setSWCapabilitiesList')).toContainEqual([['rpi_connect']])
    expect(await callsOf('removePersistedCustomisationSetting')).toHaveLength(5)
    const w = wizard()
    expect(w.selectedOsName).toBe(piOs.name)
    expect(w).toMatchObject({
      customizationSupported: true,
      piConnectAvailable: true,
      secureBootAvailable: false,
      ccRpiAvailable: true,
      ifAndFeaturesAvailable: true,
      ifSerial: 'Disabled',
    })
    expect(w.customizationSettings).toEqual({})
    expect(w.currentStep).toBe(Step.OS)
    fireEvent.click(next())
    expect(wizard().currentStep).toBe(Step.Storage)
  })

  it('Enter on an image selects it and advances', async () => {
    const { wizard } = await atOsStep()
    const list = await mainListbox()
    fireEvent.keyDown(list, { key: 'ArrowDown' })
    fireEvent.keyDown(list, { key: 'Enter' })
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Storage))
    expect(wizard().selectedOsName).toBe(piOs.name)
  })

  it('double-click selects and advances', async () => {
    const { wizard } = await atOsStep()
    fireEvent.doubleClick(screen.getByText(piOs.name))
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Storage))
  })
})

describe('OSSelectionStep groups', () => {
  it('opens a group with a Back row and returns with Left or Back', async () => {
    await atOsStep()
    fireEvent.click(screen.getByText(group.name))
    const sub = await screen.findByRole('listbox', { name: /^Operating system category\. 1 / })
    expect(screen.getByText('Go back to main menu')).toBeInTheDocument()
    expect(await screen.findByText('Cached on your computer')).toBeInTheDocument()
    expect(await callsOf('getSublist')).toEqual([[group]])
    fireEvent.keyDown(sub, { key: 'ArrowLeft' })
    const main = await mainListbox()
    expect(screen.getAllByRole('option')[1]).toHaveAttribute('aria-selected', 'true')
    fireEvent.keyDown(main, { key: 'ArrowRight' })
    await screen.findByRole('listbox', { name: /^Operating system category/ })
    fireEvent.click(screen.getByText('Go back to main menu'))
    await mainListbox()
  })

  it('an image picked in a group carries the category and Right ignores images', async () => {
    const { wizard } = await atOsStep()
    const main = await mainListbox()
    fireEvent.keyDown(main, { key: 'ArrowDown' })
    fireEvent.keyDown(main, { key: 'ArrowRight' })
    expect(await callsOf('getSublist')).toHaveLength(0)
    fireEvent.click(screen.getByText(group.name))
    await screen.findByRole('listbox', { name: /^Operating system category/ })
    fireEvent.click(screen.getByText('Lite'))
    await waitFor(() => expect(wizard().selectedOsName).toBe('Lite'))
    expect((await callsOf('setSrc')).at(-1)?.[0]).toMatchObject({ parentCategory: 'Other' })
    expect(next()).toBeEnabled()
  })
})
