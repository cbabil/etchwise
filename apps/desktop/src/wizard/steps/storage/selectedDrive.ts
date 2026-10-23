/**
 * Backend-side storage selection the wizard state does not hold: the
 * device path last passed to `setDst` (upstream `ImageWriter::_dst`), and
 * the pending "Storage device removed" notice of `main.qml`, which must
 * survive the jump back to the Storage step.
 */
import { useSyncExternalStore } from 'react'

import { call } from '@/api/imager'

export interface SelectedDrive {
  device: string
  size: number
}

let selected: SelectedDrive | null = null
let removedNotice = false
const listeners = new Set<() => void>()

const notify = () => listeners.forEach((listener) => listener())

/** `imageWriter.setDst(device, size)`, remembering the device. */
export function setDst(device: string, size: number): void {
  selected = device ? { device, size } : null
  void call('setDst', device, size)
}

export const selectedDrive = (): SelectedDrive | null => selected

/** Opens (or closes) the "Storage device removed" dialog. */
export function setRemovedNotice(open: boolean): void {
  removedNotice = open
  notify()
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

export function useRemovedNotice(): boolean {
  return useSyncExternalStore(subscribe, () => removedNotice)
}
