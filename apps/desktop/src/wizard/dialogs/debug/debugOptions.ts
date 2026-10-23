/** Debug option values (backend `getDebugOptions` / `setDebugOptions`) and their status summary. */
import { useEffect, useState } from 'react'

import { call } from '@/api/imager'

export interface DebugOptions {
  directIo: boolean
  periodicSync: boolean
  verboseLogging: boolean
  asyncIo: boolean
  asyncQueueDepth: number
  ipv4Only: boolean
  skipEndOfDevice: boolean
  rpiboot: boolean
}

export const defaultDebugOptions: DebugOptions = {
  directIo: false,
  periodicSync: false,
  verboseLogging: false,
  asyncIo: false,
  asyncQueueDepth: 16,
  ipv4Only: false,
  skipEndOfDevice: false,
  rpiboot: false,
}

/** Upstream slider snap points (1 to 512). */
export const QUEUE_DEPTH_SNAPS = [1, 2, 4, 8, 16, 32, 64, 128, 256, 384, 512]

export function snapQueueDepth(value: number): number {
  return QUEUE_DEPTH_SNAPS.reduce((best, snap) =>
    Math.abs(value - snap) < Math.abs(value - best) ? snap : best,
  )
}

const onOff = (on: boolean) => (on ? 'Enabled' : 'Disabled')

/** Upstream "Current Status" text (untranslated there too), without its symbol glyphs. */
export function statusLines(o: DebugOptions): string[] {
  const depth = Math.round(o.asyncQueueDepth)
  const lines = [
    `Direct I/O: ${onOff(o.directIo)}`,
    `Async I/O: ${o.asyncIo ? `Enabled (depth ${depth}, ~${depth}-${depth * 8} MB)` : 'Disabled'}`,
    `Periodic Sync: ${onOff(o.periodicSync)}`,
    `IPv4-only: ${onOff(o.ipv4Only)}`,
    `Counterfeit Card Mode: ${onOff(o.skipEndOfDevice)}`,
    `Rpiboot/Fastboot: ${onOff(o.rpiboot)}`,
  ]
  if (o.directIo && o.asyncIo) lines.push('Optimal: Direct I/O + Async I/O for best performance')
  else if (o.directIo) lines.push('Consider enabling Async I/O to improve Direct I/O performance')
  if (o.directIo && o.periodicSync)
    lines.push('Note: Periodic sync is skipped when Direct I/O is active')
  if (o.ipv4Only)
    lines.push('Note: Image downloads will use IPv4 only (OS list fetching unaffected)')
  if (o.skipEndOfDevice) lines.push('Counterfeit mode: Skipping end-of-device checks')
  return lines
}

/** Loads the options every time the dialog opens; `apply` sends them back. */
export function useDebugOptions(open: boolean) {
  const [options, setOptions] = useState<DebugOptions>(defaultDebugOptions)
  useEffect(() => {
    if (!open) return
    let live = true
    void call<Partial<DebugOptions> | null>('getDebugOptions').then((loaded) => {
      if (live) setOptions({ ...defaultDebugOptions, ...(loaded ?? {}) })
    })
    return () => {
      live = false
    }
  }, [open])
  const change = (patch: Partial<DebugOptions>) => setOptions((o) => ({ ...o, ...patch }))
  const apply = () =>
    void call('setDebugOptions', {
      ...options,
      asyncQueueDepth: Math.round(options.asyncQueueDepth),
    })
  return { options, change, apply }
}
