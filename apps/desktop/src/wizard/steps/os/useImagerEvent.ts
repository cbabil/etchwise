/** Subscribes a component to backend signals (upstream `Connections { target: imageWriter }`). */
import { useEffect, useRef } from 'react'

import { type ImagerEvent, onImagerEvent } from '@/api/imager'

export function useImagerEvent(handler: (event: ImagerEvent) => void): void {
  const latest = useRef(handler)
  useEffect(() => {
    latest.current = handler
  })
  useEffect(() => {
    let off: (() => void) | null = null
    let live = true
    void onImagerEvent((event) => latest.current(event)).then((unsubscribe) => {
      if (live) off = unsubscribe
      else unsubscribe()
    })
    return () => {
      live = false
      off?.()
    }
  }, [])
}
