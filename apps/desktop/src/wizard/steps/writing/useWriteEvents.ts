/**
 * Backend signals the Writing step reacts to (upstream `Connections` on
 * `imageWriter`), and the 100 ms `progress` poll upstream runs while a
 * write is in progress.
 */
import { useEffect, useRef } from 'react'

import { call, onImagerEvent } from '@/api/imager'

import { parseSnapshot, type ProgressSnapshot } from './writeProgress'

export const PROGRESS_POLL_MS = 100

export type WriteEventHandlers = Partial<Record<string, (data: unknown) => void>>

/** Subscribes to backend events by name for the lifetime of the component. */
export function useWriteEvents(handlers: WriteEventHandlers): void {
  const ref = useRef(handlers)
  useEffect(() => {
    ref.current = handlers
  })
  useEffect(() => {
    let off: (() => void) | null = null
    let live = true
    void onImagerEvent((event) => ref.current[event.name]?.(event.data)).then((unsubscribe) => {
      if (live) off = unsubscribe
      else unsubscribe()
    })
    return () => {
      live = false
      off?.()
    }
  }, [])
}

/** Polls `progress` every 100 ms while `enabled`. */
export function useProgressPoll(enabled: boolean, onSnapshot: (p: ProgressSnapshot) => void) {
  const ref = useRef(onSnapshot)
  useEffect(() => {
    ref.current = onSnapshot
  })
  useEffect(() => {
    if (!enabled) return
    let live = true
    const timer = setInterval(() => {
      void call('progress').then((value) => {
        if (live) ref.current(parseSnapshot(value))
      })
    }, PROGRESS_POLL_MS)
    return () => {
      live = false
      clearInterval(timer)
    }
  }, [enabled])
}

/** Write throughput in KB/s, measured from the counters about once a second. */
export function useThroughput(): (p: ProgressSnapshot, now: number) => number {
  const sample = useRef<{ at: number; bytes: number; kbps: number } | null>(null)
  return (p, now) => {
    const bytes = p.state === 'Verifying' ? p.verifyNow : p.bytesWritten
    const last = sample.current
    if (!last || bytes < last.bytes) {
      sample.current = { at: now, bytes, kbps: 0 }
    } else if (now - last.at >= 1000) {
      const kbps = (bytes - last.bytes) / 1024 / ((now - last.at) / 1000)
      sample.current = { at: now, bytes, kbps }
    }
    return sample.current?.kbps ?? 0
  }
}
