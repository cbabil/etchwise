/** Upstream's "no write in progress" check on `imageWriter.writeState`. */
import { call } from '@/api/imager'

const IDLE_STATES = ['Idle', 'Succeeded', 'Failed', 'Cancelled']

/** True when the backend is not writing (an unknown state counts as idle). */
export async function isWriteIdle(): Promise<boolean> {
  const state = await call<string | null>('writeState')
  return state == null || IDLE_STATES.includes(state)
}
