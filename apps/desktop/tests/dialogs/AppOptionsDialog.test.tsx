import { fireEvent, screen, waitFor, within } from '@testing-library/react'
import { useState } from 'react'

import { AppOptionsDialog } from '@/wizard/dialogs/AppOptionsDialog'
import { pickFile } from '@/wizard/dialogs/common/pickFile'

import { callsOf, setAnswers } from './imagerMock'
import { renderInWizard } from './renderWizard'

vi.mock('@/api/imager', () => import('./imagerMock'))
vi.mock('@/wizard/dialogs/common/pickFile', () => ({ pickFile: vi.fn() }))

function Harness() {
  const [open, setOpen] = useState(true)
  return (
    <>
      <button onClick={() => setOpen(true)}>reopen</button>
      <AppOptionsDialog open={open} onClose={() => setOpen(false)} />
    </>
  )
}

const settings: Record<string, unknown> = { beep: true, eject: true }

function backend(extra: Record<string, (...a: unknown[]) => unknown> = {}) {
  setAnswers({
    isBeepAvailable: () => true,
    getBoolSetting: (key) => settings[String(key)] ?? false,
    getStringSetting: () => '',
    hasWindowDecorations: () => true,
    constantVersion: () => 'v1.0.0',
    writeState: () => 'Idle',
    ...extra,
  })
}

const sw = (name: string) => screen.getByRole('switch', { name: new RegExp(`^${name}`) })
const dialog = () => screen.getByRole('dialog', { name: 'App Options' })

async function openOptions(initial = {}) {
  const result = await renderInWizard(<Harness />, { initial })
  await waitFor(() => expect(sw('Eject media when finished')).toBeChecked())
  return result
}

describe('AppOptionsDialog', () => {
  it('loads the saved options and saves the edited ones', async () => {
    backend()
    await openOptions()
    expect(sw('Play sound when finished')).toBeChecked()
    expect(screen.queryByRole('switch', { name: /statistics|telemetry/i })).toBeNull()
    expect(screen.queryByRole('link', { name: 'What is this?' })).toBeNull()
    expect(screen.queryByText(/^Version:/)).toBeNull()
    expect(screen.queryByRole('button', { name: /^Secure Boot RSA Key/ })).toBeNull()
    fireEvent.click(sw('Play sound when finished'))
    fireEvent.click(sw('Eject media when finished'))
    fireEvent.click(within(dialog()).getByRole('button', { name: /^Save,/ }))
    expect(callsOf('setSetting')).toEqual([
      ['beep', false],
      ['eject', false],
      ['secureboot_rsa_key', ''],
    ])
  })

  it('disables the sound switch when no audio player exists and shows the version', async () => {
    backend({
      isBeepAvailable: () => false,
      hasWindowDecorations: () => false,
      isEmbeddedMode: () => true,
    })
    await openOptions()
    expect(sw('Play sound when finished')).toBeDisabled()
    expect(sw('Play sound when finished')).not.toBeChecked()
    expect(screen.getByText('Version: v1.0.0')).toBeInTheDocument()
    expect(screen.queryByRole('link', { name: 'What is this?' })).toBeNull()
  })

  it('confirms before disabling warnings and reverts on Cancel', async () => {
    backend()
    const { wizard } = await openOptions()
    fireEvent.click(sw('Disable warnings'))
    const confirm = screen.getByRole('dialog', { name: 'Disable warnings?' })
    expect(
      within(confirm).getByText('not show confirmation prompts before writing images'),
    ).toBeInTheDocument()
    fireEvent.click(within(confirm).getByRole('button', { name: /^Cancel/ }))
    await waitFor(() => expect(confirm).not.toBeInTheDocument())
    expect(sw('Disable warnings')).not.toBeChecked()
    fireEvent.click(sw('Disable warnings'))
    const again = screen.getByRole('dialog', { name: 'Disable warnings?' })
    fireEvent.click(within(again).getByRole('button', { name: /^Disable warnings/ }))
    expect(wizard().disableWarnings).toBe(true)
    await waitFor(() => expect(again).not.toBeInTheDocument())
    fireEvent.click(sw('Disable warnings'))
    expect(wizard().disableWarnings).toBe(false)
  })

  it('clears saved customisation after confirmation and closes', async () => {
    backend()
    const { wizard } = await openOptions({ hostnameConfigured: true })
    fireEvent.click(screen.getByRole('button', { name: /^Saved Customisation - Clear/ }))
    const confirm = screen.getByRole('dialog', { name: 'Clear saved customisation?' })
    fireEvent.click(within(confirm).getByRole('button', { name: /^Cancel/ }))
    await waitFor(() => expect(confirm).not.toBeInTheDocument())
    fireEvent.click(screen.getByRole('button', { name: /^Saved Customisation - Clear/ }))
    fireEvent.click(
      within(screen.getByRole('dialog', { name: 'Clear saved customisation?' })).getByRole(
        'button',
        {
          name: /^Clear/,
        },
      ),
    )
    expect(callsOf('clearSavedCustomisationSettings')).toHaveLength(1)
    expect(wizard().hostnameConfigured).toBe(false)
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'App Options' })).toBeNull())
  })

  it('picks the Secure Boot key when secure boot is available and saves it', async () => {
    backend({ getStringSetting: () => '/old.pem' })
    vi.mocked(pickFile).mockResolvedValueOnce('/new.pem')
    await openOptions({ secureBootAvailable: true })
    const key = screen.getByRole('button', { name: /^Secure Boot RSA Key - Change/ })
    fireEvent.click(key)
    await waitFor(() => expect(pickFile).toHaveBeenCalled())
    await waitFor(() => {
      fireEvent.click(within(dialog()).getByRole('button', { name: /^Save,/ }))
      expect(callsOf('setSetting')).toContainEqual(['secureboot_rsa_key', '/new.pem'])
    })
  })

  it('disables repository and key editing while writing', async () => {
    backend({ writeState: () => 'Writing', checkSWCapability: () => true })
    await openOptions()
    expect(screen.getByRole('button', { name: /^Content Repository - Edit/ })).toBeDisabled()
    expect(screen.getByRole('button', { name: /^Secure Boot RSA Key - Select/ })).toBeDisabled()
  })

  it('opens the repository dialog from Edit and closes on Cancel', async () => {
    backend()
    await openOptions()
    fireEvent.click(screen.getByRole('button', { name: /^Content Repository - Edit/ }))
    const repo = await screen.findByRole('dialog', { name: 'Content Repository' })
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'App Options' })).toBeNull())
    fireEvent.click(within(repo).getByRole('button', { name: /^Cancel/ }))
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
  })
})
