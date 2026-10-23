/**
 * Upstream's progress section: a 16px bold centred status, the Qt
 * Material progress bar (4px track in a 20px row, accent #6cc04a over a
 * 25% tint, indeterminate when the size is unknown), the grey speed line
 * and the amber operation warning.
 */
import WarningAmberIcon from '@mui/icons-material/WarningAmber'
import { Box, LinearProgress } from '@mui/material'

import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

import type { useWriteJob } from './useWriteJob'
import { barValue, progressText, speedLine } from './writeProgress'

type Job = ReturnType<typeof useWriteJob>

/** Qt Material ProgressBar track: accent at 25% opacity. */
const trackTint = 'rgba(108, 192, 74, 0.25)'

function Bar({ job }: { job: Job }) {
  const tr = useTr('WritingStep')
  const { progress, extras } = job
  const indeterminate =
    extras.indeterminate && progress.state !== 'Verifying' && progress.state !== 'Finalizing'
  return (
    <Box sx={{ height: spacing.large, display: 'flex', alignItems: 'center' }}>
      <LinearProgress
        aria-label={tr('Write progress')}
        variant={indeterminate ? 'indeterminate' : 'determinate'}
        value={barValue(progress, extras.cancelRequested)}
        sx={{
          flex: 1,
          height: 4,
          borderRadius: 0,
          backgroundColor: trackTint,
          '& .MuiLinearProgress-bar': {
            backgroundColor: colors.progressBarVerifyForeground,
            transition: 'none',
          },
        }}
      />
    </Box>
  )
}

const centred = { width: '100%', textAlign: 'center' } as const

export function ProgressSection({ job }: { job: Job }) {
  const tr = useTr('WritingStep')
  const writer = useTr('ImageWriter')
  const { progress, extras, active } = job
  const status =
    extras.bottleneck || (progress.state === 'Verifying' ? writer('Verifying written data') : '')
  const speed = speedLine(status, extras.bottleneckKBps || job.measuredKBps)
  const text = progressText({ ...extras, progress }, tr)
  return (
    <Box
      sx={{
        width: '100%',
        maxWidth: sizes.sectionMaxWidth,
        mx: 'auto',
        display: 'flex',
        flexDirection: 'column',
        gap: `${spacing.medium}px`,
      }}
    >
      <Box role="status" sx={{ ...centred, fontSize: fontSize.md, fontWeight: 700 }}>
        {text}
      </Box>
      {active ? <Bar job={job} /> : null}
      {active && speed ? (
        <Box sx={{ ...centred, fontSize: fontSize.xs, color: colors.formLabelDisabled }}>
          {speed}
        </Box>
      ) : null}
      {active && extras.warning && progress.state !== 'Verifying' ? (
        <Box sx={{ ...centred, fontSize: fontSize.xs, color: colors.writeWarning }}>
          <WarningAmberIcon aria-hidden sx={{ fontSize: fontSize.xs, verticalAlign: '-1px' }} />{' '}
          {extras.warning}
        </Box>
      ) : null}
    </Box>
  )
}
