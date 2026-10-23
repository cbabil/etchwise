/**
 * The backend `progress` snapshot (upstream `ImageWriter::writeState`
 * plus the progress counters) and the texts and bar value upstream
 * `WritingStep.qml` derives from them.
 */
import { z } from 'zod'

export const writeStates = [
  'Idle',
  'Preparing',
  'Writing',
  'Verifying',
  'Finalizing',
  'Succeeded',
  'Failed',
  'Cancelling',
  'Cancelled',
] as const

export type WriteState = (typeof writeStates)[number]

const snapshotSchema = z.object({
  state: z.enum(writeStates).catch('Idle'),
  downloadNow: z.number().catch(0),
  downloadTotal: z.number().catch(0),
  bytesWritten: z.number().catch(0),
  writeTotal: z.number().catch(0),
  verifyNow: z.number().catch(0),
  verifyTotal: z.number().catch(0),
})

export type ProgressSnapshot = z.infer<typeof snapshotSchema>

export const idleSnapshot: ProgressSnapshot = {
  state: 'Idle',
  downloadNow: 0,
  downloadTotal: 0,
  bytesWritten: 0,
  writeTotal: 0,
  verifyNow: 0,
  verifyTotal: 0,
}

export function parseSnapshot(value: unknown): ProgressSnapshot {
  const parsed = snapshotSchema.safeParse(value)
  return parsed.success ? parsed.data : idleSnapshot
}

/** Upstream `isWriting`. */
export const isActive = (s: WriteState): boolean =>
  s === 'Preparing' ||
  s === 'Writing' ||
  s === 'Verifying' ||
  s === 'Finalizing' ||
  s === 'Cancelling'

export const isTerminal = (s: WriteState): boolean =>
  s === 'Succeeded' || s === 'Failed' || s === 'Cancelled'

const percent = (now: number, total: number): number => (total > 0 ? (now / total) * 100 : 0)

/** Progress bar value 0…100. */
export function barValue(p: ProgressSnapshot, cancelRequested: boolean): number {
  if (cancelRequested || p.state === 'Finalizing' || p.state === 'Cancelling') return 100
  if (p.state === 'Verifying') return percent(p.verifyNow, p.verifyTotal)
  if (p.state === 'Writing') return percent(p.bytesWritten, p.writeTotal)
  return 0
}

export interface TextInputs {
  progress: ProgressSnapshot
  indeterminate: boolean
  cancelRequested: boolean
  /** Last `preparationStatusUpdate` message, shown while preparing. */
  preparation: string
}

type Tr = (source: string, ...args: unknown[]) => string

/** Upstream `progressText.text` for the current snapshot. */
export function progressText(i: TextInputs, tr: Tr): string {
  const p = i.progress
  if (p.state === 'Succeeded') return tr('Write completed successfully!')
  if (i.cancelRequested || p.state === 'Cancelling') return tr('Finalising…')
  if (p.state === 'Finalizing') return tr('Finalising...')
  if (p.state === 'Verifying')
    return tr('Verifying... %1%', Math.round(percent(p.verifyNow, p.verifyTotal)))
  if (p.state === 'Writing') {
    if (i.indeterminate)
      return tr('Writing... %1 MB written', Math.round(p.bytesWritten / (1024 * 1024)))
    return tr('Writing... %1%', Math.round(percent(p.bytesWritten, p.writeTotal)))
  }
  return i.preparation || tr('Starting write process...')
}

/** Upstream `bottleneckText`: "<status> (N MB/s)", or empty. */
export function speedLine(status: string, throughputKBps: number): string {
  if (!status) return ''
  if (throughputKBps > 0) return `${status} (${Math.round(throughputKBps / 1024)} MB/s)`
  return status
}
