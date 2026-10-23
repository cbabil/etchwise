/**
 * Upstream `WizardStepBase.qml`: 24px margins (10px at the bottom), a
 * 24px bold title with optional 14px subtitle, the step content filling
 * the middle (scrolling internally), and a bottom row with Skip on the
 * left and Back (white) / Next (red) on the right. `customButtons`
 * replaces the whole standard row.
 */
import { Box } from '@mui/material'
import type { ReactNode } from 'react'

import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { MarqueeText } from '@/components/MarqueeText'
import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

export interface StepBaseProps {
  title: string
  subtitle?: string
  nextLabel?: string
  backLabel?: string
  skipLabel?: string
  nextEnabled?: boolean
  backEnabled?: boolean
  skipEnabled?: boolean
  showBack?: boolean
  showNext?: boolean
  showSkip?: boolean
  nextAccessibleDescription?: string
  backAccessibleDescription?: string
  skipAccessibleDescription?: string
  onNext?: () => void
  onBack?: () => void
  onSkip?: () => void
  customButtons?: ReactNode
  children?: ReactNode
}

function Header({ title, subtitle }: { title: string; subtitle: string | undefined }) {
  if (!title && !subtitle) return null
  return (
    <Box
      sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.small}px`, flexShrink: 0 }}
    >
      {title ? (
        <MarqueeText
          component="h1"
          text={title}
          sx={{ fontSize: fontSize.xl, fontWeight: 700, color: colors.formLabel, lineHeight: 1.2 }}
        />
      ) : null}
      {subtitle ? (
        <MarqueeText
          component="p"
          text={subtitle}
          sx={{ fontSize: fontSize.sm, color: colors.textDescription }}
        />
      ) : null}
    </Box>
  )
}

/** Qt Layout min / max width: grows to fit its label up to 1.5x, shrinks to min when tight. */
const capped = (min: number) => ({ minWidth: min, maxWidth: min * 1.5, flex: '0 1 auto' })

function StandardButtons(props: StepBaseProps) {
  const tr = useTr('WizardStepBase')
  const common = useTr('CommonStrings')
  const { showBack = true, showNext = true, showSkip = false } = props
  return (
    <>
      {showSkip ? (
        <ImButton
          onClick={props.onSkip}
          disabled={props.skipEnabled === false}
          {...(props.skipAccessibleDescription
            ? { accessibleDescription: props.skipAccessibleDescription }
            : {})}
          sx={capped(sizes.buttonSkipWidth)}
        >
          {props.skipLabel ?? tr('Skip customisation')}
        </ImButton>
      ) : null}
      <Box sx={{ flex: 1 }} />
      {showBack ? (
        <ImButton
          onClick={props.onBack}
          disabled={props.backEnabled === false}
          {...(props.backAccessibleDescription
            ? { accessibleDescription: props.backAccessibleDescription }
            : {})}
          sx={capped(sizes.buttonMinWidth)}
        >
          {props.backLabel ?? common('Back')}
        </ImButton>
      ) : null}
      {showNext ? (
        <ImButtonRed
          onClick={props.onNext}
          disabled={props.nextEnabled === false}
          {...(props.nextAccessibleDescription
            ? { accessibleDescription: props.nextAccessibleDescription }
            : {})}
          sx={capped(sizes.buttonMinWidth)}
        >
          {props.nextLabel ?? tr('Next')}
        </ImButtonRed>
      ) : null}
    </>
  )
}

export function StepBase(props: StepBaseProps) {
  const margin = `${sizes.stepContentMargins}px`
  return (
    <Box
      sx={{
        height: '100%',
        boxSizing: 'border-box',
        display: 'flex',
        flexDirection: 'column',
        gap: `${sizes.stepContentSpacing}px`,
        pt: margin,
        px: margin,
        pb: `${spacing.small}px`,
        minWidth: 0,
      }}
    >
      <Header title={props.title} subtitle={props.subtitle} />
      <Box
        sx={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column', overflowY: 'auto' }}
      >
        {props.children}
      </Box>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: `${spacing.small}px`, flexShrink: 0 }}>
        {props.customButtons ?? <StandardButtons {...props} />}
      </Box>
    </Box>
  )
}
