/**
 * The editable part of Kura's disk layout: the root, /tmp (in RAM unless it
 * has its own volume) and swap sizes, then one row per volume and "Add a
 * folder". Read-only for the Server preset.
 */
import { Box } from '@mui/material'

import { ImButton } from '@/components/ImButton'
import { useTr } from '@/i18n'
import { spacing } from '@/theme/tokens'

import { DescriptionText } from '../custom/layout'
import { FolderRow } from './FolderRow'
import { addRow, DISK_CONTEXT, type Layout, tmpOnDisk, type VolumeRow } from './layout'
import { SizeRow } from './SizeField'

interface Changes {
  onChange: (l: Layout) => void
}

/** The root, /tmp (or a note when it has its own volume) and swap sizes. */
function FixedSizes({ layout, locked, onChange }: { layout: Layout; locked: boolean } & Changes) {
  const tr = useTr(DISK_CONTEXT)
  return (
    <>
      <SizeRow
        caption={tr('Root (/)')}
        label={tr('Root size in GiB')}
        value={layout.root}
        disabled={locked}
        onChange={(root) => onChange({ ...layout, root })}
      />
      {tmpOnDisk(layout) ? (
        <DescriptionText>{tr('/tmp is on its own volume.')}</DescriptionText>
      ) : (
        <SizeRow
          caption={tr('/tmp (in RAM)')}
          label={tr('/tmp size in GiB')}
          value={layout.tmp}
          disabled={locked}
          onChange={(tmp) => onChange({ ...layout, tmp })}
        />
      )}
      <SizeRow
        caption={tr('Swap')}
        label={tr('Swap size in GiB')}
        value={layout.swap}
        disabled={locked}
        onChange={(swap) => onChange({ ...layout, swap })}
      />
    </>
  )
}

export function VolumeRows({ layout, onChange }: { layout: Layout } & Changes) {
  const tr = useTr(DISK_CONTEXT)
  const locked = layout.preset !== 'custom'
  const setRow = (index: number) => (row: VolumeRow | null) =>
    onChange({
      ...layout,
      volumes: row
        ? layout.volumes.map((v, i) => (i === index ? row : v))
        : layout.volumes.filter((_, i) => i !== index),
    })
  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.small}px` }}>
      <FixedSizes layout={layout} locked={locked} onChange={onChange} />
      {layout.volumes.map((row, i) => (
        // By position: the folder is edited as text, so it cannot be the key
        <FolderRow key={i} row={row} locked={locked} set={setRow(i)} />
      ))}
      {locked ? null : (
        <Box>
          <ImButton onClick={() => onChange(addRow(layout))}>{tr('Add a folder')}</ImButton>
        </Box>
      )}
    </Box>
  )
}
