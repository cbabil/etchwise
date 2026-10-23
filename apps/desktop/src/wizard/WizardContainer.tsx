/**
 * Upstream `WizardContainer.qml` layout: sidebar, drag handle and the
 * current step, cross-faded over 250ms whenever the step changes.
 */
import { Box } from '@mui/material'
import { useEffect, useState } from 'react'

import { AppOptionsDialog } from './dialogs/AppOptionsDialog'
import { Sidebar } from './Sidebar'
import { SidebarResizeHandle } from './SidebarResizeHandle'
import { stepComponents } from './steps'
import { useWizard } from './WizardContext'

export const STEP_FADE_MS = 250

/** Shows `step` after fading the previous one out, then fades it in. */
function useFadedStep(step: number): { shown: number; visible: boolean } {
  const [shown, setShown] = useState(step)
  useEffect(() => {
    if (step === shown) return
    const timer = setTimeout(() => setShown(step), STEP_FADE_MS)
    return () => clearTimeout(timer)
  }, [step, shown])
  return { shown, visible: step === shown }
}

export function WizardContainer() {
  const wizard = useWizard()
  const [optionsOpen, setOptionsOpen] = useState(false)
  const { shown, visible } = useFadedStep(wizard.currentStep)
  const Current = stepComponents[shown]
  return (
    <Box sx={{ display: 'flex', height: '100%', width: '100%', overflow: 'hidden' }}>
      <Sidebar onOpenAppOptions={() => setOptionsOpen(true)} />
      <SidebarResizeHandle />
      <Box
        component="main"
        data-step={shown}
        sx={{
          flex: 1,
          minWidth: 0,
          height: '100%',
          opacity: visible ? 1 : 0,
          transition: `opacity ${STEP_FADE_MS}ms`,
        }}
      >
        {wizard.ready && Current ? <Current key={shown} /> : null}
      </Box>
      <AppOptionsDialog open={optionsOpen} onClose={() => setOptionsOpen(false)} />
    </Box>
  )
}
