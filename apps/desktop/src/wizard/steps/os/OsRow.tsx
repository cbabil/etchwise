/**
 * Upstream `OSSelectionStep.qml` delegate: icon, bold name, description
 * (with the localised " (Recommended)" suffix), the cache / local / online
 * status line and "Released: …". The built-in Erase and Use custom rows
 * are translated by URL like upstream's `main` context strings.
 */
import { formatSize } from '@/format'
import { useTr } from '@/i18n'

import { ListRow, MetaLine } from './ListRow'
import { isBackRow, type OsEntry, URL_CUSTOM, URL_FORMAT } from './types'

const ROW_MIN_HEIGHT = 80

export interface OsRowContext {
  /** Hashes `isCached` answered true for. */
  cached: ReadonlySet<string>
  /** Size of the chosen custom image, 0 until one is picked. */
  customSize: number
}

export function useSizeFormatter(): (bytes: number) => string {
  const tr = useTr('ImageWriter')
  return (bytes) => formatSize(bytes, 1, [tr('B'), tr('KB'), tr('MB'), tr('GB'), tr('TB')])
}

/** Display name and description, translated for the built-in rows. */
export function useEntryText(): (entry: OsEntry) => { name: string; description: string } {
  const main = useTr('main')
  const model = useTr('OSListModel')
  const step = useTr('OSSelectionStep')
  const common = useTr('CommonStrings')
  return (entry) => {
    if (isBackRow(entry)) return { name: common('Back'), description: step('Go back to main menu') }
    if (entry.url === URL_FORMAT || entry.url === URL_CUSTOM)
      return { name: main(entry.name), description: main(entry.description) }
    const suffix = entry.recommended ? ` (${model('Recommended')})` : ''
    return { name: entry.name, description: entry.description + suffix }
  }
}

function useStatusText(): (entry: OsEntry, ctx: OsRowContext) => string | null {
  const tr = useTr('OSSelectionStep')
  const size = useSizeFormatter()
  return (entry, ctx) => {
    const url = entry.url ?? ''
    if (url === URL_FORMAT) return null
    if (url === URL_CUSTOM)
      return ctx.customSize > 0 ? tr('Local - %1', size(ctx.customSize)) : null
    if (!url) return ''
    if (entry.extract_sha256 && ctx.cached.has(entry.extract_sha256))
      return tr('Cached on your computer')
    if (url.startsWith('file://')) return tr('Local file')
    return tr('Online - %1 download', size(entry.image_download_size ?? 0))
  }
}

export function OsRow({
  entry,
  current,
  ctx,
}: {
  entry: OsEntry
  current: boolean
  ctx: OsRowContext
}) {
  const tr = useTr('OSSelectionStep')
  const text = useEntryText()(entry)
  const status = useStatusText()(entry, ctx)
  const released = entry.release_date ?? ''
  return (
    <ListRow
      current={current}
      minHeight={ROW_MIN_HEIGHT}
      icon={entry.icon}
      name={text.name}
      description={text.description}
    >
      {status === null ? null : <MetaLine>{status}</MetaLine>}
      {released ? <MetaLine>{tr('Released: %1', released)}</MetaLine> : null}
    </ListRow>
  )
}
