import { open } from '@tauri-apps/plugin-dialog'
import { act, fireEvent, screen, waitFor } from '@testing-library/react'

import { mockEmit } from '@/api/mock'
import { applyCustomImage, toFileUrl } from '@/wizard/steps/os/customImage'
import { resetUpdatePopupShown } from '@/wizard/steps/os/useListMetadata'
import { OSSelectionStep } from '@/wizard/steps/OSSelectionStep'

import { mainList, piOs } from './osFixtures'
import { answers, resetAnswers } from './selectionStepImager'
import { callsOf, renderStep } from './stepTestBackend'

vi.mock('@/api/imager', async () => (await import('./selectionStepImager')).fakeImager())
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }))

const tauri = window as unknown as Record<string, unknown>

beforeEach(() => {
  resetUpdatePopupShown()
  resetAnswers({ getOSList: () => mainList, imagerMetadata: () => ({}) })
})
afterEach(() => {
  delete tauri.__TAURI_INTERNALS__
})

const main = () => screen.findByRole('listbox', { name: /^Operating system list/ })
const next = () => screen.getByRole('button', { name: 'Next' })

describe('Erase', () => {
  it('formats with no customisation or capabilities', async () => {
    answers.isSecureBootForcedByCliFlag = () => true
    const { wizard } = renderStep(<OSSelectionStep />)
    await main()
    fireEvent.click(screen.getByText('Erase'))
    await waitFor(() => expect(wizard().selectedOsName).toBe('Erase'))
    expect((await callsOf('setSrc')).at(-1)?.[0]).toMatchObject({ url: 'internal://format' })
    expect(await callsOf('setSWCapabilitiesList')).toContainEqual([[]])
    expect(wizard()).toMatchObject({
      customizationSupported: false,
      piConnectAvailable: false,
      secureBootAvailable: true,
      ccRpiAvailable: false,
      ifAndFeaturesAvailable: false,
    })
    expect(next()).toBeEnabled()
  })
})

describe('Use custom', () => {
  it('picks a file, names it, drops customisation and shows its size', async () => {
    tauri.__TAURI_INTERNALS__ = {}
    vi.mocked(open).mockResolvedValue('/images/my.img')
    resetAnswers({
      getOSList: () => mainList,
      imagerMetadata: () => ({}),
      srcFileName: () => 'my.img',
      getSelectedSourceSize: () => 2048,
      imageSupportsCustomization: () => false,
    })
    const { wizard } = renderStep(<OSSelectionStep />, { hostnameConfigured: true })
    await main()
    fireEvent.click(screen.getByText('Use custom'))
    await screen.findByText('Local - 2 KB')
    expect(vi.mocked(open).mock.calls[0]?.[0]).toMatchObject({
      title: 'Select image',
      filters: [
        { name: 'Image files (*.img *.zip *.iso *.gz *.xz *.zst *.wic)' },
        { name: 'All files (*)', extensions: ['*'] },
      ],
    })
    expect(await callsOf('setSrc')).toContainEqual([{ url: 'file:///images/my.img' }])
    expect(wizard()).toMatchObject({
      selectedOsName: 'my.img',
      customizationSupported: false,
      hostnameConfigured: false,
      piConnectAvailable: false,
    })
    expect(next()).toBeEnabled()
    expect(screen.getAllByRole('option')[3]).toHaveAttribute('aria-selected', 'true')
  })

  it('keeps Next disabled when the dialog is cancelled or unavailable', async () => {
    renderStep(<OSSelectionStep />, { selectedOsName: 'x' })
    await main()
    fireEvent.click(screen.getByText('Use custom'))
    await waitFor(() => expect(next()).toBeDisabled())
    expect(open).not.toHaveBeenCalled()
  })

  it('builds file URLs and keeps customisation for capable images', async () => {
    expect(toFileUrl('C:\\img\\a.img')).toBe('file:///C:/img/a.img')
    expect(toFileUrl('file:///a.img')).toBe('file:///a.img')
    answers.imageSupportsCustomization = () => true
    const wizard = {
      setSelectedOsName: vi.fn(),
      update: vi.fn(),
      removeCustomizationSetting: vi.fn(),
    }
    await expect(applyCustomImage(wizard, '/a.img')).resolves.toBe(0)
    expect(wizard.update).toHaveBeenCalledWith({ customizationSupported: true })
  })
})

describe('offline', () => {
  it('shows the banner with Retry and reloads when the list arrives', async () => {
    resetAnswers({
      getOSList: () => mainList.slice(2),
      isOsListUnavailable: () => true,
      imagerMetadata: () => ({}),
    })
    renderStep(<OSSelectionStep />)
    await screen.findByText('Unable to download OS list. You can still use a local image file.')
    fireEvent.click(screen.getByRole('button', { name: /^Retry/ }))
    expect(await callsOf('beginOSListFetch')).toHaveLength(1)
    answers.isOsListUnavailable = () => false
    answers.getOSList = () => mainList
    act(() => mockEmit({ name: 'osListPrepared', data: null }))
    await screen.findByText(piOs.name)
    act(() => mockEmit({ name: 'osListUnavailableChanged', data: null }))
    await waitFor(() => expect(screen.queryByText(/Unable to download/)).toBeNull())
    act(() => mockEmit({ name: 'cacheStatusChanged', data: null }))
  })
})

describe('repository metadata', () => {
  it('offers a newer Imager once and opens the download page', async () => {
    answers.imagerMetadata = () => ({ latest_version: '9.0', url: 'https://rpi/imager' })
    answers.isVersionNewer = () => true
    renderStep(<OSSelectionStep />)
    await screen.findByText(/Imager version 9\.0 is available/)
    fireEvent.click(screen.getByRole('button', { name: /^Update/ }))
    expect(await callsOf('openUrl')).toEqual([['https://rpi/imager']])
    await waitFor(() => expect(screen.queryByText(/Imager version/)).toBeNull())
    renderStep(<OSSelectionStep />)
    await waitFor(async () => expect(await callsOf('imagerMetadata')).toHaveLength(2))
    expect(screen.queryByText(/Imager version/)).toBeNull()
  })

  it('stays quiet when check_version is off, and No closes the dialog', async () => {
    answers.imagerMetadata = () => ({ latest_version: '9.0', url: 'u' })
    answers.isVersionNewer = () => true
    answers.getBoolSetting = () => false
    renderStep(<OSSelectionStep />)
    await waitFor(async () => expect(await callsOf('getBoolSetting')).toHaveLength(1))
    expect(screen.queryByText(/Update available/)).toBeNull()
  })

  it('selects the default OS', async () => {
    answers.imagerMetadata = () => ({ default_os: piOs.name })
    const { wizard } = renderStep(<OSSelectionStep />)
    await waitFor(() => expect(wizard().selectedOsName).toBe(piOs.name))
    expect(next()).toBeEnabled()
  })

  it('picks the embedded default OS and skips the prompt in embedded mode', async () => {
    answers.imagerMetadata = () => ({
      latest_version: '9.0',
      url: 'u',
      embedded_default_os: 'Erase',
    })
    answers.isEmbeddedMode = () => true
    const { wizard } = renderStep(<OSSelectionStep />)
    await waitFor(() => expect(wizard().selectedOsName).toBe('Erase'))
    expect(screen.queryByText(/Update available/)).toBeNull()
  })

  it('stays quiet when the advertised version is not newer', async () => {
    answers.imagerMetadata = () => ({ latest_version: '1.0', url: 'u' })
    answers.isVersionNewer = () => false
    renderStep(<OSSelectionStep />)
    await waitFor(async () => expect(await callsOf('isVersionNewer')).toHaveLength(1))
    expect(screen.queryByText(/Update available/)).toBeNull()
  })
})

describe('row icons', () => {
  it('frames an icon that fails to load', async () => {
    renderStep(<OSSelectionStep />)
    await main()
    const img = document.querySelector('img[src="https://downloads/icon.png"]')
    expect(img).not.toBeNull()
    fireEvent.error(img as Element)
    expect(screen.getByTestId('icon-error')).toBeInTheDocument()
  })
})
