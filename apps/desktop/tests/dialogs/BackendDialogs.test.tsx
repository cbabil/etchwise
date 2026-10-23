import { act, fireEvent, screen, waitFor, within } from '@testing-library/react'

import { BackendDialogs } from '@/wizard/dialogs/BackendDialogs'

import { callsOf, emit, setAnswers } from './imagerMock'
import { renderInWizard } from './renderWizard'

vi.mock('@/api/imager', () => import('./imagerMock'))

const TOKEN = 'rpuak_abcdefghijkmnpqrstuvwxyz123'

async function setup() {
  setAnswers({})
  const result = await renderInWizard(<BackendDialogs />)
  await waitFor(() => expect(callsOf('getSavedCustomisationSettings')).toHaveLength(1))
  vi.useFakeTimers()
  return result
}

afterEach(() => vi.useRealTimers())

describe('TokenConflictDialog', () => {
  it('enables Replace after 1.5 s and overwrites the token', async () => {
    await setup()
    emit('connectTokenConflictDetected', TOKEN)
    const dialog = screen.getByRole('dialog', {
      name: 'Replace existing Raspberry Pi Connect token?',
    })
    expect(within(dialog).getByText(/differs from your current one/)).toBeInTheDocument()
    const replace = within(dialog).getByRole('button', { name: /^Please wait/ })
    expect(replace).toBeDisabled()
    act(() => vi.advanceTimersByTime(1500))
    fireEvent.click(within(dialog).getByRole('button', { name: /^Replace token/ }))
    expect(callsOf('overwriteConnectToken')).toEqual([[TOKEN]])
  })

  it('keeps the existing token', async () => {
    await setup()
    emit('connectTokenConflictDetected', TOKEN)
    fireEvent.click(screen.getByRole('button', { name: /^Keep existing/ }))
    act(() => vi.advanceTimersByTime(1500))
    expect(callsOf('overwriteConnectToken')).toEqual([])
  })
})

describe('RepositoryUrlDialog', () => {
  it('delays switching to a remote repository and ignores a second URL', async () => {
    const { wizard } = await setup()
    act(() => wizard().setSelectedDeviceName('Raspberry Pi 5'))
    emit('repositoryUrlReceived', 'https://example.com/repo.json')
    emit('repositoryUrlReceived', 'https://evil.example/other.json')
    const dialog = screen.getByRole('dialog', { name: 'Switch to a custom repository?' })
    expect(
      within(dialog).getByLabelText('Repository URL: https://example.com/repo.json'),
    ).toBeInTheDocument()
    expect(within(dialog).getByRole('button', { name: /^Please wait/ })).toBeDisabled()
    act(() => vi.advanceTimersByTime(1500))
    fireEvent.click(within(dialog).getByRole('button', { name: /^Switch repository/ }))
    expect(callsOf('refreshOsListFrom')).toEqual([['https://example.com/repo.json']])
    expect(wizard().selectedDeviceName).toBe('')
  })

  it('opens a local manifest at once, and Cancel keeps the repository', async () => {
    await setup()
    emit('repositoryUrlReceived', 'file:///tmp/list.rpi-imager-manifest')
    const dialog = screen.getByRole('dialog', { name: 'Open local repository file?' })
    expect(within(dialog).getByRole('button', { name: /^Open,/ })).toBeEnabled()
    fireEvent.click(within(dialog).getByRole('button', { name: /^Cancel/ }))
    act(() => vi.advanceTimersByTime(500))
    expect(callsOf('refreshOsListFrom')).toEqual([])
  })
})
