import { fireEvent, render, screen, within } from '@testing-library/react'
import { useState } from 'react'

import { filterOptions, popupHeight, wordBoundaryMatch } from '@/components/comboFilter'
import { ImComboBox } from '@/components/ImComboBox'

const cities = ['London (United Kingdom)', 'Paris (France)', 'New York', 'Bern']

function Combo({ onChange = () => undefined }: { onChange?: (i: number) => void }) {
  const [value, setValue] = useState(0)
  return (
    <ImComboBox
      options={cities}
      value={value}
      onChange={(i) => {
        setValue(i)
        onChange(i)
      }}
      ariaLabel="City"
    />
  )
}

describe('comboFilter', () => {
  it('matches prefixes and word starts only', () => {
    expect(wordBoundaryMatch('Paris (France)', 'fra')).toBe(true)
    expect(wordBoundaryMatch('Paris (France)', 'ance')).toBe(false)
    expect(wordBoundaryMatch('New York', 'york')).toBe(true)
    expect(filterOptions(cities, 'ne').map((o) => o.originalIndex)).toEqual([2])
    expect(filterOptions(cities, '')).toHaveLength(4)
    expect(filterOptions(cities, 'k').map((o) => o.text)).toEqual(['London (United Kingdom)'])
  })

  it('sizes the popup between 150 and 300px within the window', () => {
    expect(popupHeight(2, 1000)).toBe(150)
    expect(popupHeight(20, 1000)).toBe(300)
    expect(popupHeight(20, 100)).toBe(100)
    expect(popupHeight(20, 10)).toBe(80)
  })
})

describe('ImComboBox', () => {
  it('opens, filters by typing and picks with Enter', () => {
    const onChange = vi.fn()
    render(<Combo onChange={onChange} />)
    const combo = screen.getByRole('combobox', { name: 'City' })
    expect(combo).toHaveTextContent('London (United Kingdom)')
    fireEvent.keyDown(combo, { key: 'Enter' })
    const list = screen.getByRole('listbox')
    fireEvent.keyDown(list, { key: 'f' })
    fireEvent.keyDown(list, { key: 'r' })
    expect(screen.getByText('Search: "fr"')).toBeInTheDocument()
    expect(screen.getByText('1 of 4')).toBeInTheDocument()
    fireEvent.keyDown(list, { key: 'Enter' })
    expect(onChange).toHaveBeenCalledWith(1)
    expect(combo).toHaveTextContent('Paris (France)')
  })

  it('handles backspace, space, arrows, no matches and Escape', () => {
    const onChange = vi.fn()
    render(<Combo onChange={onChange} />)
    fireEvent.click(screen.getByRole('combobox'))
    const list = screen.getByRole('listbox')
    fireEvent.keyDown(list, { key: ' ' })
    fireEvent.keyDown(list, { key: 'z' })
    expect(screen.getByText('No matches')).toBeInTheDocument()
    fireEvent.keyDown(list, { key: 'Backspace' })
    fireEvent.keyDown(list, { key: 'Backspace' })
    fireEvent.keyDown(list, { key: 'n' })
    fireEvent.keyDown(list, { key: ' ' })
    expect(screen.getByText('Search: "n "')).toBeInTheDocument()
    fireEvent.keyDown(list, { key: 'ArrowDown' })
    fireEvent.keyDown(list, { key: 'ArrowUp' })
    fireEvent.keyDown(list, { key: 'Tab' })
    fireEvent.keyDown(list, { key: 'Shift' })
    fireEvent.keyDown(list, { key: 'Escape' })
    expect(onChange).not.toHaveBeenCalled()
  })

  it('picks with a click', () => {
    render(<Combo />)
    fireEvent.click(screen.getByRole('combobox'))
    fireEvent.click(within(screen.getByRole('listbox')).getByText('Bern'))
    expect(screen.getByRole('combobox')).toHaveTextContent('Bern')
  })

  it('steps with arrows and the wheel while closed, within bounds', () => {
    const onChange = vi.fn()
    render(<Combo onChange={onChange} />)
    const combo = screen.getByRole('combobox')
    fireEvent.keyDown(combo, { key: 'ArrowUp' })
    fireEvent.keyDown(combo, { key: 'ArrowDown' })
    fireEvent.wheel(combo, { deltaY: 10 })
    fireEvent.wheel(combo, { deltaY: -10 })
    fireEvent.keyDown(combo, { key: 'x' })
    expect(onChange.mock.calls.map(([i]) => i)).toEqual([1, 2, 1])
  })

  it('ignores input while disabled', () => {
    const onChange = vi.fn()
    render(<ImComboBox options={cities} value={-1} onChange={onChange} disabled indicateError />)
    const combo = screen.getByRole('combobox')
    fireEvent.click(combo)
    fireEvent.keyDown(combo, { key: 'Enter' })
    fireEvent.wheel(combo, { deltaY: 10 })
    expect(screen.queryByRole('listbox')).toBeNull()
    expect(combo).toHaveAttribute('aria-invalid', 'true')
    expect(onChange).not.toHaveBeenCalled()
  })
})
