/**
 * The 40x40 row icon of upstream's device and OS delegates: repository
 * relative paths (`icons/x.png`, `../icons/x.png`) load the bundled copy,
 * anything else loads by URL. A broken image shows upstream's 1px grey
 * frame instead.
 */
import { Box } from '@mui/material'
import { useState } from 'react'

import { colors, sizes } from '@/theme/tokens'

const bundled = import.meta.glob<string>('../../../assets/icons/*', {
  eager: true,
  query: '?url',
  import: 'default',
})

const byName: Readonly<Record<string, string>> = Object.fromEntries(
  Object.entries(bundled).map(([path, url]) => [path.slice(path.lastIndexOf('/') + 1), url]),
)

/** The URL an upstream icon reference loads from ('' when there is none). */
export function resolveIcon(icon: string | undefined): string {
  if (!icon) return ''
  const local = /^(?:\.\.\/)?icons\/([^/]+)$/.exec(icon)
  return (local && byName[local[1] as string]) || icon
}

export function ListIcon({ icon }: { icon: string | undefined }) {
  const src = resolveIcon(icon)
  const [failed, setFailed] = useState<string | null>(null)
  if (!src) return null
  const size = sizes.listIcon
  return (
    <Box sx={{ width: size, height: size, flexShrink: 0, position: 'relative' }}>
      {failed === src ? (
        <Box
          data-testid="icon-error"
          sx={{ position: 'absolute', inset: 0, border: `1px solid ${colors.titleSeparator}` }}
        />
      ) : (
        <Box
          component="img"
          src={src}
          alt=""
          aria-hidden
          onError={() => setFailed(src)}
          sx={{ width: size, height: size, objectFit: 'contain', display: 'block' }}
        />
      )}
    </Box>
  )
}
