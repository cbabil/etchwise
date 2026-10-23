/** Pointer handling of the sidebar drag handle (resize, persist on release, double-click reset). */
import { type PointerEvent, useRef, useState } from 'react'

import { sizes } from '@/theme/tokens'

import { clampSidebarWidth } from './useWizardActions'
import { useWizard } from './WizardContext'

export function useSidebarDrag() {
  const { sidebarWidth, setSidebarWidth } = useWizard()
  const drag = useRef<{ x: number; width: number; last: number } | null>(null)
  const [hover, setHover] = useState(false)
  const [pressed, setPressed] = useState(false)
  const handlers = {
    onPointerDown: (event: PointerEvent) => {
      drag.current = { x: event.clientX, width: sidebarWidth, last: sidebarWidth }
      setPressed(true)
      event.currentTarget.setPointerCapture?.(event.pointerId)
    },
    onPointerMove: (event: PointerEvent) => {
      if (!drag.current) return
      const width = clampSidebarWidth(drag.current.width + event.clientX - drag.current.x)
      drag.current = { ...drag.current, last: width }
      setSidebarWidth(width)
    },
    onPointerUp: () => {
      if (!drag.current) return
      const width = drag.current.last
      drag.current = null
      setPressed(false)
      setSidebarWidth(width, true)
    },
    onPointerEnter: () => setHover(true),
    onPointerLeave: () => setHover(false),
    onDoubleClick: () => setSidebarWidth(sizes.sidebarWidth, true),
  }
  return { handlers, active: hover || pressed }
}
