/** The grouped switches, queue-depth slider and status box of `DebugOptionsDialog.qml`. */
import { Box, Slider } from '@mui/material'

import { ImOptionPill } from '@/components/ImOptionPill'
import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

import { InfoText, stripEmoji } from '../common/dialogParts'
import { type DebugOptions, snapQueueDepth, statusLines } from './debugOptions'

export interface DebugListProps {
  options: DebugOptions
  onChange: (patch: Partial<DebugOptions>) => void
}

type BoolKey = Exclude<keyof DebugOptions, 'asyncQueueDepth'>

function Group({ children }: { children: string }) {
  return (
    <InfoText size={fontSize.sm} bold>
      {children}
    </InfoText>
  )
}

function QueueDepth({ options, onChange }: DebugListProps) {
  const tr = useTr('DebugOptionsDialog')
  const depth = Math.round(options.asyncQueueDepth)
  return (
    <Box
      sx={{
        pl: `${spacing.large}px`,
        display: 'flex',
        flexDirection: 'column',
        gap: `${spacing.xSmall}px`,
      }}
    >
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          gap: `${spacing.small}px`,
          fontSize: fontSize.xs,
        }}
      >
        <span>{tr('Queue Depth:')}</span>
        <Slider
          size="small"
          min={1}
          max={512}
          step={1}
          value={depth}
          onChange={(_, value) => onChange({ asyncQueueDepth: snapQueueDepth(Number(value)) })}
          slotProps={{ input: { 'aria-label': tr('Async queue depth: %1', depth) } }}
          sx={{ flex: 1, color: colors.formControlActive }}
        />
        <Box sx={{ minWidth: 40, textAlign: 'right', fontWeight: 700 }}>{depth}</Box>
      </Box>
      <InfoText>{tr('Buffer memory: ~%1-%2 MB (varies by system RAM)', depth, depth * 8)}</InfoText>
    </Box>
  )
}

function StatusBox({ options }: { options: DebugOptions }) {
  const tr = useTr('DebugOptionsDialog')
  return (
    <Box
      sx={{
        p: `${spacing.medium}px`,
        backgroundColor: colors.titleBackground,
        borderRadius: `${sizes.sectionBorderRadius}px`,
      }}
    >
      <InfoText bold>{tr('Current Status')}</InfoText>
      <InfoText color={colors.formLabel}>{statusLines(options).join('\n')}</InfoText>
    </Box>
  )
}

/** Upstream emoji prefix of two warnings; the translation key keeps it, the UI strips it. */
export const WARNING_SIGN = String.fromCodePoint(0x26a0, 0xfe0f)

const GROUPS: readonly { title: string; pills: readonly [BoolKey, string, string][] }[] = [
  {
    title: 'I/O Options',
    pills: [
      [
        'directIo',
        'Enable Direct I/O (F_NOCACHE / O_DIRECT)',
        'Bypass the operating system page cache for writes. Slower but ensures data goes directly to device.',
      ],
      [
        'asyncIo',
        'Enable Async I/O',
        'Queue multiple writes to overlap device latency. Improves performance with Direct I/O enabled.',
      ],
      [
        'periodicSync',
        'Enable Periodic Sync',
        'Periodically flush data to disk during writes. Automatically disabled when Direct I/O is active.',
      ],
    ],
  },
  {
    title: 'Debugging',
    pills: [
      [
        'verboseLogging',
        'Verbose Performance Logging',
        'Log detailed timing information for each write operation to help diagnose performance issues.',
      ],
    ],
  },
  {
    title: 'Network Options',
    pills: [
      [
        'ipv4Only',
        'Force IPv4-only Downloads',
        'Only use IPv4 for downloads. Enable this if you experience connection issues due to broken IPv6 routing.',
      ],
    ],
  },
  {
    title: 'Workarounds',
    pills: [
      [
        'skipEndOfDevice',
        'Counterfeit Card Mode (skip end-of-device checks)',
        "Skip operations at the end of the storage device. Enable this for counterfeit SD cards that report a fake larger capacity. The image must be smaller than the card's real capacity.",
      ],
    ],
  },
  {
    title: 'Advanced Features',
    pills: [
      [
        'rpiboot',
        'Enable Rpiboot/Fastboot Support',
        'Scan for Raspberry Pi devices in USB boot mode (rpiboot). Requires libusb.',
      ],
    ],
  },
]

/** What follows a switch: the queue-depth slider or the counterfeit warning. */
function Extra({ name, options, onChange }: DebugListProps & { name: BoolKey }) {
  const tr = useTr('DebugOptionsDialog')
  if (name === 'asyncIo' && options.asyncIo)
    return <QueueDepth options={options} onChange={onChange} />
  if (name !== 'skipEndOfDevice' || !options.skipEndOfDevice) return null
  return (
    <Box sx={{ pl: `${spacing.large}px` }}>
      <InfoText color={colors.formLabelError}>
        {stripEmoji(
          tr(
            `${WARNING_SIGN} Only enable this if your SD card reports a larger capacity than it actually has. Make sure your image is smaller than the card's real capacity!`,
          ),
        )}
      </InfoText>
    </Box>
  )
}

export function DebugOptionsList({ options, onChange }: DebugListProps) {
  const tr = useTr('DebugOptionsDialog')
  return (
    <>
      {GROUPS.map((group) => (
        <Box key={group.title} sx={{ display: 'contents' }}>
          <Group>{tr(group.title)}</Group>
          {group.pills.map(([key, label, description]) => (
            <Box key={key} sx={{ display: 'contents' }}>
              <ImOptionPill
                label={tr(label)}
                accessibleDescription={tr(description)}
                checked={options[key]}
                onToggle={(v) => onChange({ [key]: v })}
              />
              <Extra name={key} options={options} onChange={onChange} />
            </Box>
          ))}
        </Box>
      ))}
      <StatusBox options={options} />
    </>
  )
}
