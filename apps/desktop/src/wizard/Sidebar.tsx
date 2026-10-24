/**
 * The wizard sidebar of upstream `WizardContainer.qml`: "Setup steps",
 * the grouped step rows (scrolling internally, current row kept in view),
 * and the App Options button pinned to the bottom.
 */
import { Box } from '@mui/material'
import { type RefObject, useEffect, useRef } from 'react'

import { ImButton } from '@/components/ImButton'
import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

import { groupNames, rowForStep } from './sidebarModel'
import { SidebarRow } from './SidebarRow'
import { useWizard } from './WizardContext'

export interface SidebarProps {
  onOpenAppOptions: () => void
}

function useKeepCurrentInView(scroller: RefObject<HTMLDivElement | null>) {
  const wizard = useWizard()
  const row = rowForStep(wizard.currentStep, wizard)
  useEffect(() => {
    const el = scroller.current?.querySelector('[aria-current="step"]')
    el?.scrollIntoView?.({ block: 'center' })
  }, [scroller, row, wizard.currentStep])
}

function SidebarFooter({ onOpenAppOptions }: SidebarProps) {
  const tr = useTr('WizardContainer')
  return (
    <Box sx={{ flexShrink: 0, px: `${sizes.cardPadding}px`, pb: `${spacing.small}px` }}>
      <ImButton
        fullWidth
        onClick={onOpenAppOptions}
        accessibleDescription={tr(
          'Open application settings to configure sound alerts, auto-eject, and content repository',
        )}
        sx={{ minWidth: 0 }}
      >
        {tr('App Options')}
      </ImButton>
    </Box>
  )
}

export function Sidebar({ onOpenAppOptions }: SidebarProps) {
  const tr = useTr('WizardContainer')
  const wizard = useWizard()
  const scroller = useRef<HTMLDivElement>(null)
  useKeepCurrentInView(scroller)
  return (
    <Box
      component="nav"
      aria-label={tr('Setup steps')}
      sx={{
        width: wizard.sidebarWidth,
        flexShrink: 0,
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: colors.sidebarBackground,
      }}
    >
      <Box
        ref={scroller}
        sx={{ flex: 1, minHeight: 0, overflowY: 'auto', m: `${sizes.cardPadding}px` }}
      >
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.xSmall}px` }}>
          <Box
            component="h2"
            sx={{
              m: 0,
              mb: `${spacing.small}px`,
              fontSize: fontSize.md,
              fontWeight: 700,
              color: colors.sidebarTextOnInactive,
            }}
          >
            {tr('Setup steps')}
          </Box>
          {groupNames(wizard.hasNetworkConnectivity).map((name, row) => (
            <SidebarRow key={name} row={row} label={tr(name)} />
          ))}
        </Box>
      </Box>
      <SidebarFooter onOpenAppOptions={onOpenAppOptions} />
    </Box>
  )
}
