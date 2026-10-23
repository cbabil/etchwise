import { render } from '@testing-library/react'

import { customizationResetFlags } from '@/wizard/state'
import {
  listAccessibleName,
  parseDriveList,
  storageStatus,
  storageStatusMessage,
  visibleDrives,
} from '@/wizard/steps/storage/drives'
import { richText } from '@/wizard/steps/storage/richText'
import { anyCustomisations, customisationLines } from '@/wizard/steps/writing/customisations'
import { phaseOf, stepTexts } from '@/wizard/steps/writing/writeButtons'
import {
  barValue,
  idleSnapshot,
  parseSnapshot,
  progressText,
  speedLine,
} from '@/wizard/steps/writing/writeProgress'

import { drive } from './writeFlowHarness'

const tr = (s: string, ...a: unknown[]) =>
  s.replace(/%(\d)/g, (_, n: string) => String(a[Number(n) - 1]))

describe('drive list rules', () => {
  it('parses defensively', () => {
    expect(parseDriveList(null)).toEqual({ drives: [], error: '' })
    expect(parseDriveList({ drives: [{ key: 'k', device: 'd' }] }).drives[0]).toMatchObject({
      description: '',
      size: 0,
      mountpoints: [],
    })
  })

  it('derives the status of the list', () => {
    const rw = drive()
    const ro = drive({ isReadOnly: true })
    const sys = drive({ isSystem: true })
    expect(storageStatus({ drives: [rw], error: '' }, true)).toBe('ok')
    expect(storageStatus({ drives: [], error: 'x' }, true)).toBe('error')
    expect(storageStatus({ drives: [], error: '' }, true)).toBe('none')
    expect(storageStatus({ drives: [ro], error: '' }, true)).toBe('readOnlyFiltered')
    expect(storageStatus({ drives: [ro], error: '' }, false)).toBe('readOnly')
    expect(storageStatus({ drives: [sys], error: '' }, true)).toBe('hidden')
    expect(visibleDrives([rw, sys], false)).toHaveLength(2)
    // Read-only rpiboot devices stay selectable, as upstream.
    expect(
      storageStatus({ drives: [drive({ isReadOnly: true, isRpiboot: true })], error: '' }, true),
    ).toBe('ok')
  })

  it.each(['error', 'none', 'readOnlyFiltered', 'readOnly', 'hidden', 'ok'] as const)(
    'has a status message for %s',
    (status) => {
      const text = storageStatusMessage(status, 'boom', tr)
      expect(status === 'ok' ? text === '' : text.length > 0).toBe(true)
    },
  )

  it('names the list with the selectable count', () => {
    expect(listAccessibleName(0, 'x', tr)).toMatch(/No devices.*\. x$/)
    expect(listAccessibleName(1, '', tr)).toMatch(/\. 1 device\./)
  })

  it('renders <b> and drops other tags', () => {
    const { container } = render(<p>{richText('a <b>bold</b> <i>c</i>')}</p>)
    expect(container.querySelector('b')).toHaveTextContent('bold')
    expect(container).toHaveTextContent('a bold c')
  })
})

describe('write progress rules', () => {
  const base = { indeterminate: false, cancelRequested: false, preparation: '' }
  it('falls back to idle on bad snapshots', () => {
    expect(parseSnapshot('nope')).toEqual(idleSnapshot)
    expect(parseSnapshot({ state: 'Weird' }).state).toBe('Idle')
  })

  it('computes bar values and texts', () => {
    const p = { ...idleSnapshot, state: 'Cancelling' as const }
    expect(barValue(p, false)).toBe(100)
    expect(barValue({ ...idleSnapshot, state: 'Writing' }, false)).toBe(0)
    expect(barValue(idleSnapshot, false)).toBe(0)
    expect(progressText({ ...base, progress: p }, tr)).toBe('Finalising…')
    expect(progressText({ ...base, progress: { ...idleSnapshot, state: 'Succeeded' } }, tr)).toBe(
      'Write completed successfully!',
    )
    expect(
      progressText(
        { ...base, preparation: 'Unmounting', progress: { ...idleSnapshot, state: 'Preparing' } },
        tr,
      ),
    ).toBe('Unmounting')
  })

  it('formats the speed line', () => {
    expect(speedLine('', 100)).toBe('')
    expect(speedLine('Limited', 0)).toBe('Limited')
  })

  it('labels each phase', () => {
    expect(phaseOf('Succeeded', false)).toBe('complete')
    expect(phaseOf('Failed', false)).toBe('review')
    expect(stepTexts('complete', tr, tr).next).toBe('Continue')
  })
})

describe('customisation lines', () => {
  const off = { ...customizationResetFlags, customizationSupported: true }
  it('lists in upstream order and counts interfaces only on Done', () => {
    const f = {
      ...off,
      ifSerial: 'Disabled',
      if1WireEnabled: true,
      piConnectEnabled: true,
      featUsbGadgetEnabled: true,
    }
    expect(customisationLines(f)).toEqual([
      'Raspberry Pi Connect enabled',
      'USB Gadget mode enabled',
      '1-Wire enabled',
    ])
    expect(anyCustomisations({ ...off, ifSerial: 'Console' }, false)).toBe(false)
    expect(anyCustomisations({ ...off, ifSerial: 'Console' }, true)).toBe(true)
    expect(
      anyCustomisations({ ...off, localeConfigured: true, customizationSupported: false }, true),
    ).toBe(false)
  })
})
