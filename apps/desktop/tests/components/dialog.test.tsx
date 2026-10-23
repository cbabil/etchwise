import { fireEvent, render, screen } from '@testing-library/react'

import { BaseDialog } from '@/components/BaseDialog'
import { DialogBody, DialogButtons, DialogHeading } from '@/components/DialogText'
import { MarqueeText } from '@/components/MarqueeText'

describe('BaseDialog', () => {
  it('renders modal content and closes on Escape', () => {
    const onClose = vi.fn()
    render(
      <BaseDialog open onClose={onClose} title="Error">
        <DialogHeading>Error</DialogHeading>
        <DialogBody>Broken</DialogBody>
        <DialogButtons>
          <button type="button">OK</button>
        </DialogButtons>
      </BaseDialog>,
    )
    const dialog = screen.getByRole('dialog', { name: 'Error' })
    expect(screen.getByText('Broken')).toBeInTheDocument()
    fireEvent.keyDown(dialog, { key: 'Escape' })
    expect(onClose).toHaveBeenCalledOnce()
  })

  it('stays open without onClose and renders nothing when closed', () => {
    const { rerender } = render(
      <BaseDialog open>
        <span>Body</span>
      </BaseDialog>,
    )
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' })
    expect(screen.getByText('Body')).toBeInTheDocument()
    rerender(
      <BaseDialog open={false}>
        <span>Body</span>
      </BaseDialog>,
    )
  })
})

describe('MarqueeText', () => {
  it('scrolls only while hovered and truncated', () => {
    const cancel = vi.fn()
    const animate = vi.fn(() => ({ cancel }) as unknown as Animation)
    render(<MarqueeText text="A long sidebar label" component="h1" />)
    const outer = screen.getByRole('heading')
    const inner = outer.firstElementChild as HTMLElement
    inner.animate = animate
    fireEvent.mouseEnter(outer)
    expect(animate).not.toHaveBeenCalled()
    Object.defineProperty(inner, 'scrollWidth', { value: 300 })
    Object.defineProperty(outer, 'clientWidth', { value: 100 })
    fireEvent.mouseEnter(outer)
    expect(animate).toHaveBeenCalledOnce()
    fireEvent.mouseLeave(outer)
    expect(cancel).toHaveBeenCalledOnce()
  })

  it('stays static where the Web Animations API is missing', () => {
    render(<MarqueeText text="Label" />)
    const outer = screen.getByText('Label').parentElement as HTMLElement
    const inner = screen.getByText('Label')
    Object.defineProperty(inner, 'scrollWidth', { value: 300 })
    Object.defineProperty(outer, 'clientWidth', { value: 100 })
    expect(() => fireEvent.mouseEnter(outer)).not.toThrow()
    fireEvent.mouseLeave(outer)
  })
})
