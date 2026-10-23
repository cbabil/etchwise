/** Subscribes to one backend signal (upstream `Connections { target: imageWriter }`). */
import { useEffect, useRef } from 'react'

import { onImagerEvent } from '@/api/imager'

/** Calls `handler(data)` whenever the backend emits the `name` signal. */
export function useImagerEvent(name: string, handler: (data: unknown) => void): void {
  const ref = useRef(handler)
  useEffect(() => {
    ref.current = handler
  })
  useEffect(() => {
    let off: (() => void) | null = null
    let live = true
    void onImagerEvent((event) => {
      if (event.name === name) ref.current(event.data)
    }).then((unsubscribe) => {
      if (live) off = unsubscribe
      else unsubscribe()
    })
    return () => {
      live = false
      off?.()
    }
  }, [name])
}
