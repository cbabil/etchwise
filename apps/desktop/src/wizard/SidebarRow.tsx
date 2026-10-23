/**
 * One sidebar group row (40px, 4px radius, red band when active) and the
 * customisation sub-items under it (28px, 12px text, bold when
 * configured, underlined when current).
 */
import { Box } from '@mui/material'

import { MarqueeText } from '@/components/MarqueeText'
import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

import {
  rowClickable,
  type RowTone,
  rowTone,
  stepForRow,
  subStepClickable,
  subStepIsCurrent,
  subStepNavigates,
  subSteps,
  subStepsVisible,
} from './sidebarModel'
import { useWizard } from './WizardContext'

const toneColor: Record<RowTone, string> = {
  active: colors.sidebarTextOnActive,
  inactive: colors.sidebarTextOnInactive,
  disabled: colors.formLabelDisabled,
}

function SubItems() {
  const wizard = useWizard()
  const tr = useTr('WizardContainer')
  const clickable = subStepClickable(wizard)
  return (
    <Box
      sx={{
        mt: `${spacing.xxSmall}px`,
        display: 'flex',
        flexDirection: 'column',
        gap: `${spacing.xxSmall}px`,
      }}
    >
      {subSteps(wizard).map((sub) => (
        <Box
          key={sub.label}
          data-testid={`sidebar-sub-${sub.label}`}
          aria-current={subStepIsCurrent(sub, wizard) ? 'step' : undefined}
          onClick={() => clickable && subStepNavigates(sub, wizard) && wizard.jumpTo(sub.target)}
          sx={{
            height: sizes.sidebarSubItemHeight,
            borderRadius: `${sizes.sidebarItemRadius}px`,
            display: 'flex',
            alignItems: 'center',
            pl: `${spacing.medium}px`,
            pr: `${spacing.small}px`,
            cursor: clickable ? 'pointer' : 'default',
          }}
        >
          <MarqueeText
            text={tr(sub.label)}
            sx={{
              flex: 1,
              fontSize: fontSize.xs,
              fontWeight: sub.configured ? 700 : 400,
              textDecoration: subStepIsCurrent(sub, wizard) ? 'underline' : 'none',
              color: clickable ? colors.sidebarTextOnInactive : colors.formLabelDisabled,
            }}
          />
        </Box>
      ))}
    </Box>
  )
}

export function SidebarRow({ row, label }: { row: number; label: string }) {
  const wizard = useWizard()
  const tone = rowTone(row, wizard)
  const clickable = rowClickable(row, wizard)
  const target = stepForRow(row, wizard.hasNetworkConnectivity)
  return (
    <Box data-row={row}>
      <Box
        role="button"
        aria-current={tone === 'active' ? 'step' : undefined}
        aria-disabled={!clickable}
        onClick={() => clickable && wizard.jumpTo(target)}
        sx={{
          height: sizes.sidebarItemHeight,
          borderRadius: `${sizes.sidebarItemRadius}px`,
          border: '1px solid',
          borderColor: tone === 'active' ? colors.sidebarActiveBackground : 'transparent',
          backgroundColor: tone === 'active' ? colors.sidebarActiveBackground : 'transparent',
          display: 'flex',
          alignItems: 'center',
          px: `${spacing.small}px`,
          cursor: clickable ? 'pointer' : 'default',
          overflow: 'hidden',
        }}
      >
        <MarqueeText text={label} sx={{ flex: 1, fontSize: fontSize.sm, color: toneColor[tone] }} />
      </Box>
      {subStepsVisible(row, wizard) ? <SubItems /> : null}
    </Box>
  )
}
