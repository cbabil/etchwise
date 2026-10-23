import { fireEvent, render, screen } from '@testing-library/react'

import * as imager from '@/api/imager'
import { mockCall } from '@/api/mock'
import { WizardContainer } from '@/wizard/WizardContainer'
import { WizardProvider } from '@/wizard/WizardContext'

vi.mock('@/api/imager', async (original) => {
  const actual = await original<typeof import('@/api/imager')>()
  return {
    ...actual,
    call: vi.fn(async (method: string, ...args: unknown[]) =>
      method === 'isOsListUnavailable' ? true : mockCall(method, args),
    ),
  }
})

describe('offline wizard', () => {
  it('goes from Language to OS, hides Device and the OS Back button', async () => {
    render(
      <WizardProvider showLanguageSelection>
        <WizardContainer />
      </WizardProvider>,
    )
    await screen.findByRole('heading', { level: 1, name: 'Welcome' })
    expect(screen.queryByText('Device')).toBeNull()
    fireEvent.click(screen.getByRole('button', { name: 'Next' }))
    await screen.findByRole('heading', { level: 1, name: 'Choose operating system' })
    expect(screen.queryByRole('button', { name: 'Back' })).toBeNull()
    expect(imager.call).toHaveBeenCalledWith('isOsListUnavailable')
  })
})
