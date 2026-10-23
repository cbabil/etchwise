import { fireEvent, render, screen } from '@testing-library/react'

import { StepBase } from '@/wizard/StepBase'

describe('StepBase', () => {
  it('renders title, subtitle, content and the default Back / Next', () => {
    const onNext = vi.fn()
    const onBack = vi.fn()
    render(
      <StepBase
        title="Select your storage device"
        subtitle="Pick one"
        onNext={onNext}
        onBack={onBack}
      >
        <p>content</p>
      </StepBase>,
    )
    const title = screen.getByRole('heading', { level: 1, name: 'Select your storage device' })
    expect(getComputedStyle(title).fontSize).toBe('24px')
    expect(screen.getByText('Pick one')).toBeInTheDocument()
    expect(screen.getByText('content')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Skip customisation' })).toBeNull()
    fireEvent.click(screen.getByRole('button', { name: 'Next' }))
    fireEvent.click(screen.getByRole('button', { name: 'Back' }))
    expect(onNext).toHaveBeenCalledOnce()
    expect(onBack).toHaveBeenCalledOnce()
    const next = screen.getByRole('button', { name: 'Next' })
    expect(getComputedStyle(next).backgroundColor).toBe('rgb(171, 30, 58)')
    expect(getComputedStyle(screen.getByRole('button', { name: 'Back' })).backgroundColor).toBe(
      'rgb(255, 255, 255)',
    )
  })

  it('shows Skip on the left, custom labels and disabled states', () => {
    const onSkip = vi.fn()
    render(
      <StepBase
        title="Customisation"
        showSkip
        onSkip={onSkip}
        nextLabel="Write"
        backLabel="Previous"
        nextEnabled={false}
        backEnabled={false}
        nextAccessibleDescription="Start"
        backAccessibleDescription="Go back"
        skipAccessibleDescription="Skip all"
      />,
    )
    const buttons = screen.getAllByRole('button')
    expect(buttons.map((b) => b.textContent)).toEqual(['Skip customisation', 'Previous', 'Write'])
    expect(screen.getByRole('button', { name: 'Write, Start' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Previous, Go back' })).toBeDisabled()
    fireEvent.click(screen.getByRole('button', { name: 'Skip customisation, Skip all' }))
    expect(onSkip).toHaveBeenCalledOnce()
  })

  it('hides buttons, uses a custom skip label, and lets custom buttons replace the row', () => {
    const { rerender } = render(
      <StepBase
        title=""
        showBack={false}
        showNext={false}
        showSkip
        skipLabel="Later"
        skipEnabled={false}
      />,
    )
    expect(screen.queryByRole('heading')).toBeNull()
    expect(screen.getByRole('button', { name: 'Later' })).toBeDisabled()
    rerender(<StepBase title="Done" customButtons={<button type="button">Finish</button>} />)
    expect(screen.getAllByRole('button').map((b) => b.textContent)).toEqual(['Finish'])
  })
})
