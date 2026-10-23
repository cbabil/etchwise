/**
 * One storage device row of upstream `StorageSelectionStep.qml`'s
 * delegate: 80px high, 15px padding, a 40px SD / USB / disk icon, the
 * bold name, its size and mount points, and a red "Read-only" tag on
 * unselectable rows drawn at 50% opacity.
 */
import { Box } from '@mui/material'

import rpibootIcon from '@/assets/icons/ic_rpiboot_40px.svg'
import sdIcon from '@/assets/icons/ic_sd_storage_40px.svg'
import storageIcon from '@/assets/icons/ic_storage_40px.svg'
import usbIcon from '@/assets/icons/ic_usb_40px.svg'
import { MarqueeText } from '@/components/MarqueeText'
import { formatSize } from '@/format'
import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

import { type DriveInfo, unselectable } from './drives'

export const ROW_HEIGHT = 80

const iconFor = (d: DriveInfo): string =>
  d.isRpiboot ? rpibootIcon : d.isUsb ? usbIcon : d.isScsi ? storageIcon : sdIcon

function RowText({ drive }: { drive: DriveInfo }) {
  const tr = useTr('StorageSelectionStep')
  const off = unselectable(drive)
  return (
    <Box sx={{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column', gap: '2px' }}>
      <MarqueeText
        text={drive.description}
        sx={{
          fontSize: fontSize.sm,
          fontWeight: 700,
          color: off ? colors.formLabelDisabled : colors.formLabel,
        }}
      />
      <Box
        sx={{
          fontSize: fontSize.xs,
          color: off ? colors.formLabelDisabled : colors.textDescription,
        }}
      >
        {drive.isRpiboot ? tr('Ready for USB boot') : formatSize(drive.size)}
      </Box>
      {drive.mountpoints.length > 0 ? (
        <MarqueeText
          text={tr('Mounted as %1', drive.mountpoints.join(', '))}
          sx={{
            fontSize: fontSize.xs,
            color: off ? colors.formLabelDisabled : colors.textMetadata,
          }}
        />
      ) : null}
    </Box>
  )
}

export interface StorageRowProps {
  drive: DriveInfo
  current: boolean
}

export function StorageRow({ drive, current }: StorageRowProps) {
  const tr = useTr('StorageSelectionStep')
  const off = unselectable(drive)
  return (
    <Box
      data-testid="storage-row"
      sx={{
        height: ROW_HEIGHT,
        boxSizing: 'border-box',
        p: `${sizes.listItemPadding}px`,
        display: 'flex',
        alignItems: 'center',
        gap: `${spacing.medium}px`,
        opacity: off ? 0.5 : 1,
        cursor: off ? 'not-allowed' : 'pointer',
        backgroundColor: current ? colors.listViewHighlight : colors.listViewRowBackground,
        '&:hover': off || current ? {} : { backgroundColor: colors.listViewHoverRowBackground },
      }}
    >
      <Box
        component="img"
        src={iconFor(drive)}
        alt=""
        sx={{ width: sizes.listIcon, height: sizes.listIcon, flexShrink: 0 }}
      />
      <RowText drive={drive} />
      {off ? (
        <Box sx={{ fontSize: fontSize.xs, color: colors.formLabelError, flexShrink: 0 }}>
          {tr('Read-only')}
        </Box>
      ) : null}
    </Box>
  )
}
