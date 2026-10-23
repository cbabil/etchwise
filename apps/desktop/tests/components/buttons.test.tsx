import { fireEvent, render, screen } from '@testing-library/react'

import { CloseButton } from '@/components/CloseButton'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { ImToggleTab } from '@/components/ImToggleTab'

describe('ImButton / ImButtonRed', () => {
  it('renders upstream sizes, caps and colours without ripple', () => {
    render(<ImButton>Back</ImButton>)
    const button = screen.getByRole('button', { name: 'Back' })
    const style = getComputedStyle(button)
    expect(style.height).toBe('40px')
    expect(style.minWidth).toBe('120px')
    expect(style.textTransform).toBe('uppercase')
    expect(style.borderRadius).toBe('4px')
    expect(style.backgroundColor).toBe('rgb(255, 255, 255)')
    expect(style.color).toBe('rgb(171, 30, 58)')
    expect(button.querySelector('.MuiTouchRipple-root')).toBeNull()
  })

  it('red button is raspberry red with white text', () => {
    render(<ImButtonRed>Next</ImButtonRed>)
    const style = getComputedStyle(screen.getByRole('button', { name: 'Next' }))
    expect(style.backgroundColor).toBe('rgb(171, 30, 58)')
    expect(style.color).toBe('rgb(255, 255, 255)')
  })

  it('combines the label with the accessible description and clicks', () => {
    const onClick = vi.fn()
    render(
      <ImButton accessibleDescription="Go back" onClick={onClick}>
        Back
      </ImButton>,
    )
    fireEvent.click(screen.getByRole('button', { name: 'Back, Go back' }))
    expect(onClick).toHaveBeenCalledOnce()
  })

  it('does not click while disabled', () => {
    const onClick = vi.fn()
    render(
      <ImButtonRed disabled onClick={onClick}>
        Write
      </ImButtonRed>,
    )
    fireEvent.click(screen.getByRole('button', { name: 'Write' }))
    expect(onClick).not.toHaveBeenCalled()
  })
})

describe('ImToggleTab', () => {
  it('reports its active state', () => {
    const { rerender } = render(<ImToggleTab active={false}>Tab</ImToggleTab>)
    expect(screen.getByRole('button', { name: 'Tab' })).toHaveAttribute('aria-pressed', 'false')
    rerender(<ImToggleTab active>Tab</ImToggleTab>)
    const active = screen.getByRole('button', { name: 'Tab (currently selected)' })
    expect(active).toHaveAttribute('aria-pressed', 'true')
    expect(getComputedStyle(active).backgroundColor).toBe('rgb(171, 30, 58)')
  })
})

describe('CloseButton', () => {
  it('closes on click, Enter and Space', () => {
    const onClick = vi.fn()
    render(<CloseButton onClick={onClick} />)
    const close = screen.getByRole('button', { name: 'Close' })
    expect(close).toHaveTextContent('X')
    fireEvent.click(close)
    fireEvent.keyDown(close, { key: 'Enter' })
    fireEvent.keyDown(close, { key: ' ' })
    fireEvent.keyDown(close, { key: 'a' })
    expect(onClick).toHaveBeenCalledTimes(3)
  })
})
