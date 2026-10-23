/**
 * Minimal Qt StyledText rendering for the warning dialogs: `<b>` becomes
 * bold and any other tag is dropped, so translated strings keep their
 * emphasis without injecting HTML.
 */
import type { ReactNode } from 'react'

export function richText(source: string): ReactNode[] {
  const nodes: ReactNode[] = []
  let bold = false
  source.split(/(<\/?b>|<[^>]+>)/i).forEach((part, index) => {
    if (/^<b>$/i.test(part)) bold = true
    else if (/^<\/b>$/i.test(part)) bold = false
    else if (part && !part.startsWith('<')) nodes.push(bold ? <b key={index}>{part}</b> : part)
  })
  return nodes
}
