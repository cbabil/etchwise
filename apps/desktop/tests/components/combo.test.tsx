import { fireEvent, render, screen, within } from '@testing-library/react'
import { useState } from 'react'

import {
  filterOptions,
  popupHeight,
  SEARCH_ROW_HEIGHT,
  wordBoundaryMatch,
} from '@/components/comboFilter'
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
  const search = () => screen.getByRole('searchbox', { name: 'Type to search' })

  it('opens with a focused search field that filters as you type', () => {
    const onChange = vi.fn()
    render(<Combo onChange={onChange} />)
    const combo = screen.getByRole('combobox', { name: 'City' })
    expect(combo).toHaveTextContent('London (United Kingdom)')
    fireEvent.keyDown(combo, { key: 'Enter' })
    expect(search()).toHaveFocus()
    expect(search()).toHaveAttribute('placeholder', 'Type to search')
    fireEvent.change(search(), { target: { value: 'fr' } })
    expect(within(screen.getByRole('listbox')).getAllByRole('option')).toHaveLength(1)
    expect(screen.getByText('1 of 4')).toBeInTheDocument()
    fireEvent.keyDown(search(), { key: 'Enter' })
    expect(onChange).toHaveBeenCalledWith(1)
    expect(combo).toHaveTextContent('Paris (France)')
  })

  it('moves with arrows from the search field, shows no matches, closes with Escape', () => {
    const onChange = vi.fn()
    render(<Combo onChange={onChange} />)
    fireEvent.click(screen.getByRole('combobox'))
    fireEvent.change(search(), { target: { value: 'z' } })
    expect(screen.getByText('No matches')).toBeInTheDocument()
    fireEvent.change(search(), { target: { value: 'n' } })
    fireEvent.keyDown(search(), { key: 'ArrowDown' })
    fireEvent.keyDown(search(), { key: 'ArrowDown' })
    fireEvent.keyDown(search(), { key: 'ArrowUp' })
    fireEvent.keyDown(search(), { key: 'Enter' })
    // only "New York" has a word starting with "n"
    expect(onChange).toHaveBeenCalledWith(2)
    fireEvent.click(screen.getByRole('combobox'))
    fireEvent.keyDown(search(), { key: 'Escape' })
    expect(screen.queryByRole('listbox')).toBeNull()
    expect(onChange).toHaveBeenCalledTimes(1)
  })

  it('keeps the keyboard working after a click inside the list', () => {
    const onChange = vi.fn()
    render(<Combo onChange={onChange} />)
    fireEvent.click(screen.getByRole('combobox'))
    const list = screen.getByRole('listbox')
    fireEvent.keyDown(list, { key: 'ArrowDown' })
    fireEvent.keyDown(list, { key: 'Enter' })
    expect(onChange).toHaveBeenCalledWith(1)
  })

  it('leaves Enter and the arrows to an input method while it composes', () => {
    const onChange = vi.fn()
    render(<Combo onChange={onChange} />)
    fireEvent.click(screen.getByRole('combobox'))
    fireEvent.keyDown(search(), { key: 'Enter', isComposing: true })
    fireEvent.keyDown(search(), { key: 'Enter', keyCode: 229 })
    expect(onChange).not.toHaveBeenCalled()
    expect(screen.getByRole('listbox')).toBeInTheDocument()
  })

  it('ignores spaces before the search text', () => {
    render(<Combo />)
    fireEvent.click(screen.getByRole('combobox'))
    fireEvent.change(search(), { target: { value: '  new' } })
    expect(within(screen.getByRole('listbox')).getAllByRole('option')).toHaveLength(1)
  })

  it('tells assistive technology which row is highlighted', () => {
    render(<Combo />)
    fireEvent.click(screen.getByRole('combobox'))
    const list = screen.getByRole('listbox')
    expect(search()).toHaveAttribute('aria-controls', list.id)
    fireEvent.keyDown(search(), { key: 'ArrowDown' })
    const active = search().getAttribute('aria-activedescendant')
    expect(document.getElementById(active ?? '')).toHaveTextContent('Paris (France)')
  })

  it('makes room for the search field in the popup height', () => {
    expect(popupHeight(4, 1000)).toBeGreaterThanOrEqual(4 * 40 + SEARCH_ROW_HEIGHT)
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
