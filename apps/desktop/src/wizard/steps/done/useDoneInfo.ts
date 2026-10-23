/**
 * Backend answers the Done step needs: whether the device was ejected
 * automatically (`getBoolSetting('eject')`) and embedded mode (Finish
 * becomes Reboot). Finish closes the window, as upstream `Qt.quit()`.
 */
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useEffect, useState } from 'react'

import { call } from '@/api/imager'

export interface DoneInfo {
  autoEject: boolean
  embedded: boolean
}

export function useDoneInfo(): DoneInfo {
  const [info, setInfo] = useState<DoneInfo>({ autoEject: false, embedded: false })
  useEffect(() => {
    let live = true
    void Promise.all([
      call<boolean | null>('getBoolSetting', 'eject'),
      call<boolean | null>('isEmbeddedMode'),
    ]).then(([eject, embedded]) => {
      if (live) setInfo({ autoEject: Boolean(eject), embedded: Boolean(embedded) })
    })
    return () => {
      live = false
    }
  }, [])
  return info
}

const inTauri = (): boolean => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

/** Finish: reboot in embedded mode, otherwise close the app window. */
export async function finish(embedded: boolean): Promise<void> {
  if (embedded) {
    await call('reboot')
    return
  }
  if (inTauri()) await getCurrentWindow().close()
}
