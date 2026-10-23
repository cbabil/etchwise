/**
 * Bridge to the Rust backend: `imageWriter.<method>(…)` calls from the
 * upstream QML become `call('<method>', …)`, and upstream signals arrive
 * as `{name, data}` events. Outside Tauri (browser preview, tests) a mock
 * backend answers instead.
 */
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import { mockCall, mockListen } from './mock'

export interface ImagerEvent {
  name: string
  data: unknown
}

const inTauri = (): boolean => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

/** Calls `imageWriter.<method>(args…)` on the backend. */
export async function call<T = unknown>(method: string, ...args: unknown[]): Promise<T> {
  if (!inTauri()) return mockCall(method, args) as T
  return invoke<T>('imager', { method, args })
}

/** Subscribes to backend signals; returns an unsubscribe function. */
export async function onImagerEvent(handler: (event: ImagerEvent) => void): Promise<() => void> {
  if (!inTauri()) return mockListen(handler)
  return listen<ImagerEvent>('imager://event', (e) => handler(e.payload))
}
