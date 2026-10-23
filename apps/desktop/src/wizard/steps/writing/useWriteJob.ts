/**
 * The write lifecycle of upstream `WritingStep.qml`: the 300 ms
 * `beginWriteDelay` before `startWrite`, polling, cancel / skip
 * verification, the backend signals, and auto-advance on success.
 */
import { useEffect, useRef, useState } from 'react'

import { call } from '@/api/imager'

import { useWizard } from '../../WizardContext'
import { useClearRemovedStorage } from '../storage/deviceRemoval'
import { useProgressPoll, useThroughput, useWriteEvents } from './useWriteEvents'
import {
  idleSnapshot,
  isActive,
  isTerminal,
  parseSnapshot,
  type ProgressSnapshot,
} from './writeProgress'

export const BEGIN_WRITE_DELAY_MS = 300

interface Extras {
  indeterminate: boolean
  cancelRequested: boolean
  preparation: string
  bottleneck: string
  bottleneckKBps: number
  warning: string
  startError: string
}

const freshExtras: Extras = {
  indeterminate: false,
  cancelRequested: false,
  preparation: '',
  bottleneck: '',
  bottleneckKBps: 0,
  warning: '',
  startError: '',
}

function useBeginDelay(start: () => Promise<void>) {
  const [running, setRunning] = useState(false)
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null)
  useEffect(() => () => void (timer.current && clearTimeout(timer.current)), [])
  const begin = () => {
    setRunning(true)
    timer.current = setTimeout(() => {
      setRunning(false)
      void start()
    }, BEGIN_WRITE_DELAY_MS)
  }
  return { running, begin }
}

function useEventHandlers(
  setProgress: (change: (p: ProgressSnapshot) => ProgressSnapshot) => void,
  patch: (e: Partial<Extras>) => void,
) {
  const clearRemoved = useClearRemovedStorage()
  const to = (state: ProgressSnapshot['state']) => () => setProgress((p) => ({ ...p, state }))
  useWriteEvents({
    success: to('Succeeded'),
    error: to('Failed'),
    cancelled: to('Cancelled'),
    writeCancelledDueToDeviceRemoval: () => {
      setProgress((p) => ({ ...p, state: 'Cancelled' }))
      clearRemoved(true)
    },
    preparationStatusUpdate: (data) => patch({ preparation: String(data ?? '') }),
    operationWarning: (data) => patch({ warning: String(data ?? '') }),
    bottleneckStatusChanged: (data) => {
      const d = (data ?? {}) as { status?: unknown; throughputKBps?: unknown }
      patch({ bottleneck: String(d.status ?? ''), bottleneckKBps: Number(d.throughputKBps) || 0 })
    },
  })
}

export function useWriteJob() {
  const [progress, setProgress] = useState<ProgressSnapshot>(idleSnapshot)
  const [extras, setExtras] = useState<Extras>(freshExtras)
  const [measuredKBps, setMeasured] = useState(0)
  const measure = useThroughput()
  const patch = (e: Partial<Extras>) => setExtras((x) => ({ ...x, ...e }))
  const active = isActive(progress.state)
  useEventHandlers(setProgress, patch)
  useProgressPoll(active, (p) => {
    setMeasured(measure(p, Date.now()))
    // The backend reports Idle until its write thread starts: keep Preparing.
    setProgress((old) =>
      isTerminal(old.state) ? old : p.state === 'Idle' ? { ...p, state: old.state } : p,
    )
  })
  useResumeAndSync(active, progress.state, setProgress)
  const delay = useBeginDelay(async () => {
    const known = await call<boolean | null>('isExtractSizeKnown')
    setExtras({ ...freshExtras, indeterminate: known === false })
    setProgress({ ...idleSnapshot, state: 'Preparing' })
    await call('startWrite').catch((error: unknown) => {
      setProgress({ ...idleSnapshot, state: 'Failed' })
      patch({ startError: String(error) })
    })
  })
  const cancel = () => {
    patch({ cancelRequested: true })
    void call('cancelWrite')
  }
  const skipVerification = () => void call('skipCurrentVerification')
  const clearStartError = () => patch({ startError: '' })
  return {
    progress,
    extras,
    measuredKBps,
    active,
    delay,
    cancel,
    skipVerification,
    clearStartError,
  }
}

/** Resumes a write already running on mount, mirrors `isWriting`, advances on success. */
function useResumeAndSync(
  active: boolean,
  state: ProgressSnapshot['state'],
  setProgress: (p: ProgressSnapshot) => void,
) {
  const { update, next } = useWizard()
  useEffect(() => {
    let live = true
    void call('progress').then((value) => {
      const p = parseSnapshot(value)
      if (live && isActive(p.state)) setProgress(p)
    })
    return () => {
      live = false
    }
  }, [setProgress])
  useEffect(() => update({ isWriting: active }), [active, update])
  const advanced = useRef(false)
  useEffect(() => {
    if (state !== 'Succeeded' || advanced.current) return
    advanced.current = true
    next()
  }, [state, next])
}
