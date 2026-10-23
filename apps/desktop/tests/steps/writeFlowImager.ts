/**
 * Scriptable stand-in for `@/api/imager`, used by the step tests via
 * `vi.mock('@/api/imager', () => import('./writeFlowImager').then((m) => m.imagerMock))`.
 * It imports nothing from the app, so the mock factory cannot deadlock.
 */
import type { ImagerEvent } from '@/api/imager'

type Answer = (...args: unknown[]) => unknown

export const backend = {
  answers: {} as Record<string, Answer>,
  calls: [] as Array<[string, unknown[]]>,
  handlers: new Set<(event: ImagerEvent) => void>(),
  reset() {
    this.answers = {}
    this.calls = []
    this.handlers.clear()
  },
  emit(name: string, data: unknown = null) {
    this.handlers.forEach((h) => h({ name, data }))
  },
  called(method: string): unknown[][] {
    return this.calls.filter(([m]) => m === method).map(([, args]) => args)
  },
}

export const imagerMock = {
  call: async (method: string, ...args: unknown[]) => {
    backend.calls.push([method, args])
    return backend.answers[method]?.(...args) ?? null
  },
  onImagerEvent: async (handler: (event: ImagerEvent) => void) => {
    backend.handlers.add(handler)
    return () => backend.handlers.delete(handler)
  },
}
