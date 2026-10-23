import { fireEvent, render, screen, waitFor } from '@testing-library/react'

import { SelectionList } from '@/components/SelectionList'
import { findNextSelectableIndex, stepIndex } from '@/components/selectionNav'

const items = ['A', 'header', 'B', 'C']
const selectable = (i: number) => items[i] !== 'header'

describe('selectionNav', () => {
  it('skips unselectable rows and stays at the edges', () => {
    expect(findNextSelectableIndex(0, 1, 4, selectable)).toBe(2)
    expect(findNextSelectableIndex(2, -1, 4, selectable)).toBe(0)
    expect(findNextSelectableIndex(3, 1, 4, selectable)).toBe(3)
    expect(stepIndex(-1, 'ArrowUp', 4, selectable)).toBe(0)
    expect(stepIndex(0, 'ArrowUp', 4, selectable)).toBe(0)
    expect(stepIndex(3, 'ArrowDown', 4, selectable)).toBe(3)
    expect(stepIndex(5, 'ArrowDown', 0, selectable)).toBe(5)
  })
})

describe('SelectionList', () => {
  it('navigates with arrows, selects with Enter / Space and auto-advances', async () => {
    const onItemSelected = vi.fn()
    const onNext = vi.fn()
    render(
      <SelectionList
        items={items}
        renderItem={(item) => <span>{item}</span>}
        isItemSelectable={selectable}
        onItemSelected={onItemSelected}
        keyboardAutoAdvance
        onNext={onNext}
        ariaLabel="Devices"
      />,
    )
    const list = screen.getByRole('listbox', { name: 'Devices' })
    fireEvent.focus(list)
    fireEvent.keyDown(list, { key: 'Enter' })
    expect(onItemSelected).not.toHaveBeenCalled()
    fireEvent.keyDown(list, { key: 'ArrowDown' })
    fireEvent.keyDown(list, { key: 'ArrowDown' })
    expect(screen.getAllByRole('option')[2]).toHaveAttribute('aria-selected', 'true')
    expect(getComputedStyle(screen.getAllByRole('option')[2]!).backgroundColor).toBe(
      'rgb(186, 204, 231)',
    )
    fireEvent.keyDown(list, { key: ' ' })
    expect(onItemSelected).toHaveBeenCalledWith(2, 'B')
    await waitFor(() => expect(onNext).toHaveBeenCalledOnce())
    fireEvent.keyDown(list, { key: 'ArrowUp' })
    expect(screen.getAllByRole('option')[0]).toHaveAttribute('aria-selected', 'true')
    fireEvent.keyDown(list, { key: 'x' })
    fireEvent.blur(list)
  })

  it('selects on click, ignores unselectable rows and reports double-clicks', () => {
    const onItemSelected = vi.fn()
    const onItemDoubleClicked = vi.fn()
    const onCurrentIndexChange = vi.fn()
    render(
      <SelectionList
        items={items}
        renderItem={(item) => item}
        getKey={(item) => item}
        isItemSelectable={selectable}
        currentIndex={-1}
        onCurrentIndexChange={onCurrentIndexChange}
        onItemSelected={onItemSelected}
        onItemDoubleClicked={onItemDoubleClicked}
      />,
    )
    fireEvent.click(screen.getByText('header'))
    fireEvent.click(screen.getByText('C'))
    fireEvent.doubleClick(screen.getByText('C'))
    expect(onItemSelected).toHaveBeenCalledExactlyOnceWith(3, 'C')
    expect(onCurrentIndexChange).toHaveBeenCalledWith(3)
    expect(onItemDoubleClicked).toHaveBeenCalledWith(3, 'C')
  })

  it('auto-selects the first row', () => {
    render(<SelectionList items={['x', 'y']} renderItem={(i) => i} autoSelectFirst />)
    expect(screen.getAllByRole('option')[0]).toHaveAttribute('aria-selected', 'true')
  })
})
