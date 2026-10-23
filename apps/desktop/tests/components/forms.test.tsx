import { fireEvent, render, screen } from '@testing-library/react'
import { useState } from 'react'

import * as imager from '@/api/imager'
import { ImCheckBox } from '@/components/ImCheckBox'
import { ImOptionPill } from '@/components/ImOptionPill'
import { ImPasswordField } from '@/components/ImPasswordField'
import { ImRadioButton } from '@/components/ImRadioButton'
import { ImTextField } from '@/components/ImTextField'

vi.mock('@/api/imager', async (original) => {
  const actual = await original<typeof import('@/api/imager')>()
  return { ...actual, call: vi.fn(actual.call) }
})

function Password() {
  const [value, setValue] = useState('')
  return <ImPasswordField value={value} onChange={setValue} label="Password" />
}

describe('ImCheckBox', () => {
  it('toggles on click and on Enter', () => {
    const onChange = vi.fn()
    render(
      <ImCheckBox label="Eject" checked={false} onChange={onChange} accessibleDescription="desc" />,
    )
    const box = screen.getByRole('checkbox', { name: 'Eject' })
    fireEvent.click(box)
    fireEvent.keyDown(box, { key: 'Enter' })
    expect(onChange).toHaveBeenNthCalledWith(1, true)
    expect(onChange).toHaveBeenNthCalledWith(2, true)
  })
})

describe('ImRadioButton', () => {
  it('selects with Space / Enter only when unchecked', () => {
    const onSelect = vi.fn()
    const { rerender } = render(
      <ImRadioButton label="A" checked={false} onSelect={onSelect} name="g" value="a" />,
    )
    fireEvent.keyDown(screen.getByRole('radio', { name: 'A' }), { key: ' ' })
    expect(onSelect).toHaveBeenCalledOnce()
    rerender(<ImRadioButton label="A" checked onSelect={onSelect} />)
    fireEvent.keyDown(screen.getByRole('radio', { name: 'A' }), { key: 'Enter' })
    expect(onSelect).toHaveBeenCalledOnce()
    fireEvent.click(screen.getByRole('radio', { name: 'A' }))
  })
})

describe('ImTextField', () => {
  it('renders an outlined input', () => {
    render(<ImTextField label="Hostname" defaultValue="pi" />)
    expect(screen.getByRole('textbox', { name: 'Hostname' })).toHaveValue('pi')
  })
})

describe('ImPasswordField', () => {
  it('shows the eye toggle only after typing and toggles with F2 and click', () => {
    render(<Password />)
    const input = screen.getByLabelText('Password')
    expect(input).toHaveAttribute('type', 'password')
    expect(screen.queryByRole('button')).toBeNull()
    fireEvent.keyDown(input, { key: 'F2' })
    expect(input).toHaveAttribute('type', 'password')
    fireEvent.change(input, { target: { value: 'secret' } })
    fireEvent.keyDown(input, { key: 'F2' })
    expect(input).toHaveAttribute('type', 'text')
    fireEvent.click(screen.getByRole('button', { name: /currently visible/ }))
    expect(input).toHaveAttribute('type', 'password')
  })
})

describe('ImOptionPill', () => {
  it('toggles from the switch and the label and opens the help link', () => {
    const onToggle = vi.fn()
    render(
      <ImOptionPill
        label="Telemetry"
        checked={false}
        onToggle={onToggle}
        helpLabel="What is this?"
        helpUrl="https://x.test"
      />,
    )
    fireEvent.click(screen.getByRole('switch', { name: 'Telemetry, What is this?' }))
    fireEvent.click(screen.getByText('Telemetry'))
    fireEvent.keyDown(screen.getByRole('switch'), { key: 'Enter' })
    expect(onToggle).toHaveBeenCalledTimes(3)
    const link = screen.getByRole('link', { name: 'What is this?' })
    fireEvent.click(link)
    fireEvent.keyDown(link, { key: 'Enter' })
    fireEvent.keyDown(link, { key: 'x' })
    expect(imager.call).toHaveBeenCalledWith('openUrl', 'https://x.test')
    expect(vi.mocked(imager.call).mock.calls.filter(([m]) => m === 'openUrl')).toHaveLength(2)
  })

  it('ignores label clicks when disabled', () => {
    const onToggle = vi.fn()
    render(<ImOptionPill label="SSH" checked onToggle={onToggle} disabled />)
    fireEvent.click(screen.getByText('SSH'))
    expect(onToggle).not.toHaveBeenCalled()
  })
})
