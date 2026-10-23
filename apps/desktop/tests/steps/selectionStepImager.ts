/**
 * Backend fake for the Language / Device / OS step tests: `call` answers
 * from `answers` first, then the browser mock; events go through the
 * mock's `mockEmit`. Use as `vi.mock('@/api/imager', async () =>
 * (await import('./selectionStepImager')).fakeImager())`.
 */
import { mockCall, mockListen } from '@/api/mock'

export type Answer = (...args: unknown[]) => unknown

export const answers: Record<string, Answer> = {}

export function resetAnswers(next: Record<string, Answer> = {}): void {
  Object.keys(answers).forEach((k) => delete answers[k])
  Object.assign(answers, next)
}

export const fakeImager = () => ({
  call: vi.fn(async (method: string, ...args: unknown[]) => {
    const answer = answers[method]
    return answer ? answer(...args) : mockCall(method, args)
  }),
  onImagerEvent: vi.fn(async (handler: Parameters<typeof mockListen>[0]) => mockListen(handler)),
})
