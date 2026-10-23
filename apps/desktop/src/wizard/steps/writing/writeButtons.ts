/**
 * Subtitle, Next label and accessible description of upstream
 * `WritingStep.qml` for each phase: WRITE, then CANCEL WRITE (SKIP
 * VERIFICATION while verifying), then CONTINUE.
 */
import type { WriteState } from './writeProgress'

type Tr = (source: string, ...args: unknown[]) => string

export type Phase = 'review' | 'writing' | 'verifying' | 'complete'

export function phaseOf(state: WriteState, active: boolean): Phase {
  if (active) return state === 'Verifying' ? 'verifying' : 'writing'
  return state === 'Succeeded' ? 'complete' : 'review'
}

export interface StepTexts {
  subtitle: string
  next: string
  nextDescription: string
}

export function stepTexts(phase: Phase, tr: Tr, common: Tr): StepTexts {
  switch (phase) {
    case 'verifying':
      return {
        subtitle: tr('Writing in progress — do not disconnect the storage device'),
        next: tr('Skip verification'),
        nextDescription: tr('Skip verification and finish the write process'),
      }
    case 'writing':
      return {
        subtitle: tr('Writing in progress — do not disconnect the storage device'),
        next: tr('Cancel write'),
        nextDescription: tr('Cancel the write operation and return to the summary'),
      }
    case 'complete':
      return {
        subtitle: tr('Write complete'),
        next: common('Continue'),
        nextDescription: tr('Continue to the completion screen'),
      }
    default:
      return {
        subtitle: tr('Review your choices and write the image to the storage device'),
        next: tr('Write'),
        nextDescription: tr(
          'Begin writing the image to the storage device. All existing data will be erased.',
        ),
      }
  }
}
