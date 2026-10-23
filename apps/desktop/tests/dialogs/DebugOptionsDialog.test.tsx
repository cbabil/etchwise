import { fireEvent, render, screen, waitFor } from '@testing-library/react'

import {
  defaultDebugOptions,
  snapQueueDepth,
  statusLines,
} from '@/wizard/dialogs/debug/debugOptions'
import { DebugOptionsDialog } from '@/wizard/dialogs/DebugOptionsDialog'

import { callsOf, setAnswers } from './imagerMock'

vi.mock('@/api/imager', () => import('./imagerMock'))

const sw = (name: string) => screen.getByRole('switch', { name: new RegExp(`^${name}`) })

describe('DebugOptionsDialog', () => {
  it('loads the options, edits them and applies them in one call', async () => {
    setAnswers({ getDebugOptions: () => ({ directIo: true, asyncQueueDepth: 32 }) })
    const onClose = vi.fn()
    render(<DebugOptionsDialog open onClose={onClose} />)
    expect(screen.getByText(/^These options are for debugging and testing/)).toBeInTheDocument()
    await waitFor(() => expect(sw('Enable Direct I/O')).toBeChecked())
    expect(screen.getByText(/Consider enabling Async I\/O/)).toBeInTheDocument()
    fireEvent.click(sw('Enable Async I/O'))
    expect(screen.getByText('Buffer memory: ~32-256 MB (varies by system RAM)')).toBeInTheDocument()
    fireEvent.change(screen.getByRole('slider', { name: 'Async queue depth: 32' }), {
      target: { value: 60 },
    })
    expect(screen.getByText('Buffer memory: ~64-512 MB (varies by system RAM)')).toBeInTheDocument()
    fireEvent.click(sw('Counterfeit Card Mode'))
    expect(screen.getByText(/^Only enable this if your SD card reports/)).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: /^Apply/ }))
    expect(callsOf('setDebugOptions')).toEqual([
      [
        {
          ...defaultDebugOptions,
          directIo: true,
          asyncIo: true,
          asyncQueueDepth: 64,
          skipEndOfDevice: true,
        },
      ],
    ])
    expect(onClose).toHaveBeenCalled()
  })

  it('falls back to defaults when the backend has none and cancels without saving', async () => {
    setAnswers({})
    const onClose = vi.fn()
    render(<DebugOptionsDialog open onClose={onClose} />)
    await waitFor(() => expect(sw('Enable Rpiboot/Fastboot Support')).not.toBeChecked())
    fireEvent.click(screen.getByRole('button', { name: /^Cancel/ }))
    expect(onClose).toHaveBeenCalled()
    expect(callsOf('setDebugOptions')).toEqual([])
  })

  it('summarises the status like upstream, without symbol glyphs', () => {
    const all = {
      ...defaultDebugOptions,
      directIo: true,
      asyncIo: true,
      periodicSync: true,
      ipv4Only: true,
      skipEndOfDevice: true,
      rpiboot: true,
    }
    expect(statusLines(all)).toEqual([
      'Direct I/O: Enabled',
      'Async I/O: Enabled (depth 16, ~16-128 MB)',
      'Periodic Sync: Enabled',
      'IPv4-only: Enabled',
      'Counterfeit Card Mode: Enabled',
      'Rpiboot/Fastboot: Enabled',
      'Optimal: Direct I/O + Async I/O for best performance',
      'Note: Periodic sync is skipped when Direct I/O is active',
      'Note: Image downloads will use IPv4 only (OS list fetching unaffected)',
      'Counterfeit mode: Skipping end-of-device checks',
    ])
    expect(statusLines(defaultDebugOptions)).toHaveLength(6)
    expect(snapQueueDepth(300)).toBe(256)
    expect(snapQueueDepth(1)).toBe(1)
  })
})
