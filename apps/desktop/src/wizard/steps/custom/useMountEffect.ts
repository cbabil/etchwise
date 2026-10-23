/** Runs `effect` once after the first render (upstream `Component.onCompleted`). */
import { useEffect, useRef } from 'react'

export function useMountEffect(effect: () => void | (() => void)): void {
  const ref = useRef(effect)
  useEffect(() => ref.current(), [])
}
