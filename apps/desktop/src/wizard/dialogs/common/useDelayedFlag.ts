/**
 * Upstream's one-shot safety `Timer`s ("allowAccept"): false while
 * `active` is false and for `ms` after it turns true, then true.
 */
import { useEffect, useState } from 'react'

export function useDelayedFlag(active: boolean, ms: number): boolean {
  const [elapsed, setElapsed] = useState(false)
  useEffect(() => {
    if (!active) return
    const timer = setTimeout(() => setElapsed(true), ms)
    return () => {
      clearTimeout(timer)
      setElapsed(false)
    }
  }, [active, ms])
  return active && elapsed
}
