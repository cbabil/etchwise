import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'

import { mockEmit } from '@/api/mock'
import { App, isDebugShortcut } from '@/App'

describe('App', () => {
  it('renders the wizard shell at the device step', async () => {
    render(<App />)
    expect(
      await screen.findByRole('heading', { level: 1, name: 'Select your Raspberry Pi device' }),
    ).toBeInTheDocument()
    expect(screen.getByRole('navigation', { name: 'Setup steps' })).toBeInTheDocument()
  })

  it('shows backend errors in the error dialog until Continue', async () => {
    render(<App />)
    await screen.findByRole('heading', { level: 1 })
    act(() => mockEmit({ name: 'error', data: 'Write failed<br>Try again' }))
    const dialog = await screen.findByRole('dialog', { name: 'Error' })
    expect(dialog).toHaveTextContent('Write failed Try again')
    fireEvent.click(screen.getByRole('button', { name: /^Continue/ }))
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
  })

  it('opens debug options on Cmd+Option+S / Ctrl+Alt+S only', async () => {
    render(<App />)
    await screen.findByRole('heading', { level: 1 })
    fireEvent.keyDown(window, { code: 'KeyS', altKey: false, metaKey: true })
    expect(screen.queryByRole('dialog')).toBeNull()
    fireEvent.keyDown(window, { code: 'KeyS', altKey: true, metaKey: true })
    expect(await screen.findByRole('dialog', { name: 'Debug Options' })).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: /^Apply/ }))
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
    expect(
      isDebugShortcut(new KeyboardEvent('keydown', { code: 'KeyS', ctrlKey: true, altKey: true })),
    ).toBe(true)
  })
})
