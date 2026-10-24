import { screen } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { DoneStep } from '@/wizard/steps/DoneStep'
import { selectOs } from '@/wizard/steps/os/selectImage'
import type { OsEntry } from '@/wizard/steps/os/types'
import { WritingStep } from '@/wizard/steps/WritingStep'

import { renderStep } from './writeFlowHarness'
import { backend } from './writeFlowImager'

vi.mock('@/api/imager', () => import('./writeFlowImager').then((m) => m.imagerMock))

const ERASED = "The NUC's largest disk will be erased during installation."
const TYPE_YES =
  'Boot the NUC from the stick. When the installer asks "Continue with autoinstall?", type yes.'

beforeEach(() => {
  vi.useFakeTimers()
  backend.reset()
})
afterEach(() => vi.useRealTimers())

describe('Ubuntu autoinstall notes', () => {
  it('warns in the Writing summary only for autoinstall images', async () => {
    await renderStep(<WritingStep />, Step.Writing, { autoinstall: true })
    expect(screen.getByText(ERASED)).toBeVisible()
  })

  it('shows nothing extra for other images', async () => {
    await renderStep(<WritingStep />, Step.Writing)
    expect(screen.queryByText(ERASED)).toBeNull()
    await renderStep(<DoneStep />, Step.Done)
    expect(screen.queryByText(TYPE_YES)).toBeNull()
  })

  it('tells the user to answer yes on the NUC when done', async () => {
    await renderStep(<DoneStep />, Step.Done, { autoinstall: true })
    expect(screen.getByText(TYPE_YES)).toBeVisible()
  })

  it('asks the backend whether the chosen image uses autoinstall', async () => {
    const update = vi.fn()
    const wizard = {
      update,
      setSelectedOsName: vi.fn(),
      removeCustomizationSetting: vi.fn(),
    } as unknown as Parameters<typeof selectOs>[0]
    const entry = { name: 'Ubuntu Server', url: 'https://u/s.iso' } as OsEntry
    backend.answers.imageUsesAutoinstall = () => true
    await selectOs(wizard, entry, '')
    expect(update).toHaveBeenLastCalledWith(expect.objectContaining({ autoinstall: true }))
    backend.answers.imageUsesAutoinstall = () => false
    await selectOs(wizard, entry, '')
    expect(update).toHaveBeenLastCalledWith(expect.objectContaining({ autoinstall: false }))
  })
})
