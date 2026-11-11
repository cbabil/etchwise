/**
 * The editable part of Kura's disk layout: the root and /tmp sizes, then
 * one row per volume (folder, size in GiB or "rest of the disk", remove),
 * and "Add a folder". Read-only for the Server preset.
 */
import DeleteOutline from '@mui/icons-material/DeleteOutlineOutlined'
import { Box, IconButton } from '@mui/material'

import { ImButton } from '@/components/ImButton'
import { ImCheckBox } from '@/components/ImCheckBox'
import { ImComboBox } from '@/components/ImComboBox'
import { ImTextField } from '@/components/ImTextField'
import { useTr } from '@/i18n'
import { spacing } from '@/theme/tokens'

import { FormLabel, FormRow } from '../custom/layout'
import { acceptsInput } from '../custom/validation'
import { addRow, DISK_CONTEXT, DISK_MOUNTS, type Layout, type VolumeRow } from './layout'

const SIZE_RE = /^\d{1,4}$/
const FIRST_COLUMN = 140

interface SizeFieldProps {
  label: string
  value: string
  disabled: boolean
  onChange: (value: string) => void
}

function SizeField({ label, value, disabled, onChange }: SizeFieldProps) {
  return (
    <ImTextField
      value={value}
      disabled={disabled}
      onChange={(e) => acceptsInput(SIZE_RE, e.target.value) && onChange(e.target.value)}
      slotProps={{ htmlInput: { 'aria-label': label, inputMode: 'numeric' } }}
      sx={{ width: 96 }}
    />
  )
}

/** A fixed row: a label, then its size. */
function SizeRow(props: SizeFieldProps & { caption: string }) {
  return (
    <FormRow>
      <Box sx={{ width: FIRST_COLUMN }}>
        <FormLabel isDisabled={props.disabled}>{props.caption}</FormLabel>
      </Box>
      <SizeField {...props} />
    </FormRow>
  )
}

function Row(props: { row: VolumeRow; locked: boolean; set: (row: VolumeRow | null) => void }) {
  const tr = useTr(DISK_CONTEXT)
  const { row, locked, set } = props
  return (
    <FormRow>
      <Box sx={{ width: FIRST_COLUMN }}>
        <ImComboBox
          options={DISK_MOUNTS}
          value={DISK_MOUNTS.indexOf(row.mount)}
          disabled={locked}
          ariaLabel={tr('Folder')}
          onChange={(i) => set({ ...row, mount: DISK_MOUNTS[i] ?? row.mount })}
        />
      </Box>
      <SizeField
        label={tr('%1 size in GiB', row.mount)}
        value={row.size}
        disabled={locked || row.rest}
        onChange={(size) => set({ ...row, size })}
      />
      <ImCheckBox
        label={tr('Rest of the disk')}
        ariaLabel={tr('%1 takes the rest', row.mount)}
        checked={row.rest}
        disabled={locked}
        onChange={(rest) => set({ ...row, rest, size: '' })}
      />
      {locked ? null : (
        <IconButton disableRipple aria-label={tr('Remove %1', row.mount)} onClick={() => set(null)}>
          <DeleteOutline fontSize="small" />
        </IconButton>
      )}
    </FormRow>
  )
}

export function VolumeRows({
  layout,
  onChange,
}: {
  layout: Layout
  onChange: (l: Layout) => void
}) {
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
      <SizeRow
        caption={tr('Root (/)')}
        label={tr('Root size in GiB')}
        value={layout.root}
        disabled={locked}
        onChange={(root) => onChange({ ...layout, root })}
      />
      <SizeRow
        caption={tr('/tmp (in RAM)')}
        label={tr('/tmp size in GiB')}
        value={layout.tmp}
        disabled={locked}
        onChange={(tmp) => onChange({ ...layout, tmp })}
      />
      {layout.volumes.map((row, i) => (
        <Row key={`${row.mount}-${i}`} row={row} locked={locked} set={setRow(i)} />
      ))}
      {locked || layout.volumes.length >= DISK_MOUNTS.length ? null : (
        <Box>
          <ImButton onClick={() => onChange(addRow(layout))}>{tr('Add a folder')}</ImButton>
        </Box>
      )}
    </Box>
  )
}
