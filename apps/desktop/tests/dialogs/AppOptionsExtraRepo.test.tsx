import { fireEvent, screen, waitFor, within } from '@testing-library/react'

import { AppOptionsDialog } from '@/wizard/dialogs/AppOptionsDialog'
import { pickFile } from '@/wizard/dialogs/common/pickFile'
import { Step } from '@/wizard/stepIds'

import { callsOf, setAnswers } from './imagerMock'
import { renderInWizard } from './renderWizard'

vi.mock('@/api/imager', () => import('./imagerMock'))
vi.mock('@/wizard/dialogs/common/pickFile', () => ({ pickFile: vi.fn() }))

function backend(extraRepo: string, extraRepoError = '') {
  setAnswers({
    getBoolSetting: (key) => key === 'eject',
    getStringSetting: (key) => (key === 'extraRepo' ? extraRepo : ''),
    getExtraRepoError: () => extraRepoError,
    hasWindowDecorations: () => true,
    writeState: () => 'Idle',
  })
}

const dialog = () => screen.getByRole('dialog', { name: 'App Options' })
const extraButton = (action: string) =>
  screen.findByRole('button', { name: new RegExp(`^Extra Repository - ${action}`) })
const save = () => fireEvent.click(within(dialog()).getByRole('button', { name: /^Save,/ }))

async function open() {
  const result = await renderInWizard(<AppOptionsDialog open onClose={() => undefined} />, {
    step: Step.OS,
  })
  await waitFor(() => expect(screen.getByRole('switch', { name: /^Eject media/ })).toBeChecked())
  return result
}

describe('App Options extra repository', () => {
  it('chooses a file, saves it and reloads the lists from the start', async () => {
    backend('')
    vi.mocked(pickFile).mockResolvedValueOnce('/Users/me/repos/intel nuc.json')
    const { wizard } = await open()
    fireEvent.click(await extraButton('Choose'))
    expect(await screen.findByText('intel nuc.json')).toBeInTheDocument()
    save()
    expect(callsOf('setSetting')).toContainEqual([
      'extraRepo',
      'file:///Users/me/repos/intel%20nuc.json',
    ])
    await waitFor(() => expect(callsOf('beginOSListFetch')).toHaveLength(1))
    await waitFor(() => expect(wizard().currentStep).toBe(Step.Device))
  })

  it('removes a saved extra repository', async () => {
    backend('file:///r/nuc.json')
    await open()
    expect(await screen.findByText('nuc.json')).toBeInTheDocument()
    fireEvent.click(await extraButton('Remove'))
    save()
    expect(callsOf('setSetting')).toContainEqual(['extraRepo', ''])
    await waitFor(() => expect(callsOf('beginOSListFetch')).toHaveLength(1))
  })

  it('leaves the lists alone when the choice did not change', async () => {
    backend('file:///r/nuc.json')
    await open()
    await extraButton('Remove')
    save()
    expect(callsOf('setSetting').some(([key]) => key === 'extraRepo')).toBe(false)
    expect(callsOf('beginOSListFetch')).toHaveLength(0)
  })

  it('shows why the extra repository did not load', async () => {
    backend('file:///r/nuc.json', 'No such file or directory')
    await open()
    expect(
      await screen.findByText('Could not load nuc.json: No such file or directory'),
    ).toBeInTheDocument()
  })
})
