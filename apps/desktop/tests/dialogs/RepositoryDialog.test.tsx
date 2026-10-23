import { fireEvent, screen, waitFor } from '@testing-library/react'

import { pickFile } from '@/wizard/dialogs/common/pickFile'
import { displayUrl, fileUrlOf } from '@/wizard/dialogs/repository/useRepository'
import { RepositoryDialog } from '@/wizard/dialogs/RepositoryDialog'

import { callsOf, setAnswers } from './imagerMock'
import { renderInWizard } from './renderWizard'

vi.mock('@/api/imager', () => import('./imagerMock'))
vi.mock('@/wizard/dialogs/common/pickFile', () => ({ pickFile: vi.fn() }))

const apply = () => screen.getByRole('button', { name: /^Apply & Restart/ })
const radio = (name: string) => screen.getByRole('radio', { name: new RegExp(`^${name}`) })

function backend(repo: string | null, extra: Record<string, (...a: unknown[]) => unknown> = {}) {
  setAnswers({
    customRepo: () => repo !== null,
    osListUrl: () => repo ?? 'https://downloads.raspberrypi.com/os_list_imagingutility_v4.json',
    isValidRepoUrl: (url) => /^https:\/\/.+\.json$/.test(String(url)),
    ...extra,
  })
}

async function renderDialog(onClose = vi.fn()) {
  const result = await renderInWizard(<RepositoryDialog open onClose={onClose} />)
  return { ...result, onClose }
}

describe('RepositoryDialog', () => {
  it('keeps the official repository without refreshing', async () => {
    backend(null)
    const { onClose } = await renderDialog()
    expect(radio('Raspberry Pi \\(default\\)')).toBeChecked()
    fireEvent.click(apply())
    expect(callsOf('refreshOsListFrom')).toEqual([])
    expect(callsOf('refreshOsListFromDefaultUrl')).toEqual([])
    expect(onClose).toHaveBeenCalled()
  })

  it('switches to a valid custom URL and restarts the wizard', async () => {
    backend(null)
    const { wizard } = await renderDialog()
    fireEvent.click(radio('Use custom URL'))
    const field = screen.getByRole('textbox', { name: 'Use custom URL' })
    fireEvent.change(field, { target: { value: 'ftp://nope' } })
    await waitFor(() => expect(apply()).toBeDisabled())
    fireEvent.change(field, { target: { value: 'https://example.com/repo.json' } })
    await waitFor(() => expect(apply()).toBeEnabled())
    fireEvent.click(apply())
    expect(callsOf('refreshOsListFrom')).toEqual([['https://example.com/repo.json']])
    expect(wizard().selectedDeviceName).toBe('')
  })

  it('needs a picked file before applying a custom file', async () => {
    backend(null)
    vi.mocked(pickFile).mockResolvedValueOnce('/home/pi/my list.json')
    await renderDialog()
    fireEvent.click(radio('Use custom file'))
    expect(apply()).toBeDisabled()
    fireEvent.click(screen.getByRole('button', { name: /^Browse/ }))
    await waitFor(() =>
      expect(screen.getByRole('textbox', { name: 'Use custom file' })).toHaveValue(
        '/home/pi/my list.json',
      ),
    )
    fireEvent.click(apply())
    expect(callsOf('refreshOsListFrom')).toEqual([['file:///home/pi/my%20list.json']])
  })

  it('restores a custom file repository and switches back to the default', async () => {
    backend('file:///srv/list.json')
    await renderDialog()
    await waitFor(() => expect(radio('Use custom file')).toBeChecked())
    expect(screen.getByRole('textbox', { name: 'Use custom file' })).toHaveValue('/srv/list.json')
    fireEvent.click(radio('Raspberry Pi \\(default\\)'))
    fireEvent.click(apply())
    expect(callsOf('refreshOsListFromDefaultUrl')).toHaveLength(1)
  })

  it('restores a custom URL and does nothing when it is unchanged', async () => {
    backend('https://example.com/a.json')
    await renderDialog()
    await waitFor(() => expect(radio('Use custom URL')).toBeChecked())
    await waitFor(() => expect(apply()).toBeEnabled())
    fireEvent.click(apply())
    expect(callsOf('refreshOsListFrom')).toEqual([])
  })

  it('treats an unknown custom scheme as the default and blocks Apply while writing', async () => {
    backend('ftp://odd', { writeState: () => 'Verifying' })
    await renderDialog()
    expect(radio('Raspberry Pi \\(default\\)')).toBeChecked()
    await waitFor(() => expect(apply()).toBeDisabled())
  })

  it('converts between paths and file URLs', () => {
    expect(fileUrlOf('C:\\Users\\pi\\a.json')).toBe('file:///C:/Users/pi/a.json')
    expect(displayUrl('file:///C:/Users/pi/a.json')).toBe('C:/Users/pi/a.json')
    expect(displayUrl('https://x/y.json')).toBe('https://x/y.json')
  })
})
