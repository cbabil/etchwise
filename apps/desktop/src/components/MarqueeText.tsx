/**
 * Upstream `MarqueeText.qml`: single-line text that elides with "…" and,
 * while hovered and truncated, scrolls left to reveal the rest (pause
 * 500ms, 40px/s, pause 1500ms, 300ms return), looping.
 */
import { Box } from '@mui/material'
import type { SxProps, Theme } from '@mui/material/styles'
import { useRef, useState } from 'react'

const SCROLL_SPEED = 40
const PAUSE_START = 500
const PAUSE_END = 1500
const RETURN_MS = 300
const PADDING = 8

export interface MarqueeTextProps {
  text: string
  sx?: SxProps<Theme>
  /** Rendered element, e.g. `h1` for step titles. */
  component?: 'div' | 'span' | 'h1' | 'h2' | 'p'
  align?: 'left' | 'center' | 'right'
}

function startScroll(outer: HTMLElement, inner: HTMLElement): Animation | null {
  const distance = inner.scrollWidth - outer.clientWidth + PADDING
  if (typeof inner.animate !== 'function') return null
  const scroll = Math.max(100, (distance / SCROLL_SPEED) * 1000)
  const total = PAUSE_START + scroll + PAUSE_END + RETURN_MS
  const at = (ms: number) => ms / total
  return inner.animate(
    [
      { transform: 'translateX(0)', offset: 0 },
      { transform: 'translateX(0)', offset: at(PAUSE_START) },
      { transform: `translateX(${-distance}px)`, offset: at(PAUSE_START + scroll) },
      { transform: `translateX(${-distance}px)`, offset: at(PAUSE_START + scroll + PAUSE_END) },
      { transform: 'translateX(0)', offset: 1 },
    ],
    { duration: total, iterations: Infinity },
  )
}

export function MarqueeText({ text, sx, component = 'div', align = 'left' }: MarqueeTextProps) {
  const outer = useRef<HTMLElement>(null)
  const inner = useRef<HTMLSpanElement>(null)
  const animation = useRef<Animation | null>(null)
  const [scrolling, setScrolling] = useState(false)
  const onEnter = () => {
    if (!outer.current || !inner.current) return
    if (inner.current.scrollWidth <= outer.current.clientWidth) return
    setScrolling(true)
    animation.current = startScroll(outer.current, inner.current)
  }
  const onLeave = () => {
    animation.current?.cancel()
    animation.current = null
    setScrolling(false)
  }
  return (
    <Box
      ref={outer}
      component={component}
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
      sx={
        [
          { m: 0, overflow: 'hidden', whiteSpace: 'nowrap', textAlign: align, minWidth: 0 },
          ...(Array.isArray(sx) ? sx : sx ? [sx] : []),
        ] as SxProps<Theme>
      }
    >
      <Box
        component="span"
        ref={inner}
        sx={{
          display: 'block',
          overflow: scrolling ? 'visible' : 'hidden',
          textOverflow: scrolling ? 'clip' : 'ellipsis',
          font: 'inherit',
        }}
      >
        {text}
      </Box>
    </Box>
  )
}
