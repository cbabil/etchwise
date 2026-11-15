/**
 * One volume of Kura's disk layout: its folder (a suggestion or any folder
 * typed in, checked as Kura would), its size in GiB or "rest of the disk",
 * and remove.
 */
import DeleteOutline from '@mui/icons-material/DeleteOutlineOutlined'
import { Autocomplete, Box, IconButton } from '@mui/material'

import { ImCheckBox } from '@/components/ImCheckBox'
import { ImTextField } from '@/components/ImTextField'
import { useTr } from '@/i18n'

import { FormRow } from '../custom/layout'
import { DISK_CONTEXT, mountError, SUGGESTED_MOUNTS, type VolumeRow } from './layout'
import { FIRST_COLUMN, SizeField } from './SizeField'

interface FolderFieldProps {
  mount: string
  disabled: boolean
  onChange: (mount: string) => void
}

function FolderField({ mount, disabled, onChange }: FolderFieldProps) {
  const tr = useTr(DISK_CONTEXT)
  const error = mountError(mount)
  return (
    <Autocomplete
      freeSolo
      disableClearable
      options={SUGGESTED_MOUNTS}
      value={mount}
      inputValue={mount}
      disabled={disabled}
      onInputChange={(_, text) => onChange(text)}
      onChange={(_, choice) => onChange(choice ?? '')}
      renderInput={(params) => (
        <ImTextField
          {...params}
          error={error !== ''}
          helperText={error ? tr(error) : undefined}
          slotProps={{
            ...params.slotProps,
            htmlInput: { ...params.slotProps.htmlInput, 'aria-label': tr('Folder') },
          }}
        />
      )}
    />
  )
}

export function FolderRow(props: {
  row: VolumeRow
  locked: boolean
  set: (row: VolumeRow | null) => void
}) {
  const tr = useTr(DISK_CONTEXT)
  const { row, locked, set } = props
  const name = row.mount || tr('New folder')
  return (
    <FormRow>
      <Box sx={{ width: FIRST_COLUMN }}>
        <FolderField
          mount={row.mount}
          disabled={locked}
          onChange={(mount) => set({ ...row, mount })}
        />
      </Box>
      <SizeField
        label={tr('%1 size in GiB', name)}
        value={row.size}
        disabled={locked || row.rest}
        onChange={(size) => set({ ...row, size })}
      />
      <ImCheckBox
        label={tr('Rest of the disk')}
        ariaLabel={tr('%1 takes the rest', name)}
        checked={row.rest}
        disabled={locked}
        onChange={(rest) => set({ ...row, rest, size: '' })}
      />
      {locked ? null : (
        <IconButton disableRipple aria-label={tr('Remove %1', name)} onClick={() => set(null)}>
          <DeleteOutline fontSize="small" />
        </IconButton>
      )}
    </FormRow>
  )
}
