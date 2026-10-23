import { render, screen } from '@testing-library/react'

import * as mock from '@/api/mock'
import { WizardContainer } from '@/wizard/WizardContainer'
import { WizardProvider } from '@/wizard/WizardContext'

afterEach(() => vi.restoreAllMocks())

describe('wizard start-up', () => {
  it('still shows the first step when a start-up call fails', async () => {
    const real = mock.mockCall
    vi.spyOn(mock, 'mockCall').mockImplementation((method, args) =>
      method === 'isEmbeddedMode'
        ? Promise.reject(new Error('unknown imageWriter method: isEmbeddedMode'))
        : real(method, args),
    )
    render(
      <WizardProvider>
        <WizardContainer />
      </WizardProvider>,
    )
    expect(
      await screen.findByRole('heading', { level: 1, name: 'Select your Raspberry Pi device' }),
    ).toBeInTheDocument()
  })
})
