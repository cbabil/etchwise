/** `vi.mock('@/api/imager', () => import('./imagerMock'))`: scripted backend for step and dialog tests. */
import { act } from '@testing-library/react'
import { vi } from 'vitest'

import type { ImagerEvent } from '@/api/imager'

type Handler = (event: ImagerEvent) => void
type Answer = (...args: unknown[]) => unknown

const handlers = new Set<Handler>()
let answers: Record<string, Answer> = {}

export const call = vi.fn(
  async (method: string, ...args: unknown[]) => answers[method]?.(...args) ?? null,
)

export const onImagerEvent = vi.fn(async (handler: Handler) => {
  handlers.add(handler)
  return () => {
    handlers.delete(handler)
  }
})

/** Replaces the backend's answers (method name to function of its arguments). */
export function setAnswers(next: Record<string, Answer>): void {
  answers = next
}

/** Delivers a backend signal to every subscriber. */
export function emit(name: string, data?: unknown): void {
  act(() => handlers.forEach((h) => h({ name, data })))
}

/** Arguments of every `call(method, …)` so far. */
export const callsOf = (method: string): unknown[][] =>
  call.mock.calls.filter(([m]) => m === method).map(([, ...args]) => args)
