import { screen } from '@testing-library/react'

import { AppOptionsDialog } from '@/wizard/dialogs/AppOptionsDialog'

import { setAnswers } from './imagerMock'
import { renderInWizard } from './renderWizard'

vi.mock('@/api/imager', () => import('./imagerMock'))

describe('App Options credit', () => {
  it('names the app and version, and credits Raspberry Pi Imager', async () => {
    setAnswers({
      constantVersion: () => 'v1.0.0',
      hasWindowDecorations: () => true,
      writeState: () => 'Idle',
    })
    await renderInWizard(<AppOptionsDialog open onClose={() => undefined} />)
    expect(
      await screen.findByText('Etchwise v1.0.0 · Based on Raspberry Pi Imager (Apache-2.0)'),
    ).toBeInTheDocument()
  })
})
