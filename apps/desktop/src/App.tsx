/**
 * Window shell of upstream `main.qml`: translations, the wizard, the
 * global error dialog and the Cmd+Option+S (Ctrl+Alt+S) debug shortcut.
 */
import { Box } from '@mui/material'
import { useCallback, useEffect, useState } from 'react'

import { AppWatchers } from './AppWatchers'
import { I18nProvider } from './i18n'
import { colors } from './theme/tokens'
import { DebugOptionsDialog } from './wizard/dialogs/DebugOptionsDialog'
import { ErrorDialog } from './wizard/dialogs/ErrorDialog'
import { WizardContainer } from './wizard/WizardContainer'
import { WizardProvider } from './wizard/WizardContext'

/** Qt "Ctrl+Alt+S": Cmd+Option+S on macOS, Ctrl+Alt+S elsewhere. */
export function isDebugShortcut(event: KeyboardEvent): boolean {
  return (event.metaKey || event.ctrlKey) && event.altKey && event.code === 'KeyS'
}

function useDebugShortcut(open: () => void) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!isDebugShortcut(event)) return
      event.preventDefault()
      open()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [open])
}

export function App() {
  const [debugOpen, setDebugOpen] = useState(false)
  useDebugShortcut(useCallback(() => setDebugOpen(true), []))
  return (
    <I18nProvider>
      <WizardProvider>
        <Box
          sx={{
            height: '100vh',
            width: '100vw',
            overflow: 'hidden',
            backgroundColor: colors.mainBackground,
          }}
        >
          <WizardContainer />
        </Box>
        <ErrorDialog />
        <AppWatchers />
        <DebugOptionsDialog open={debugOpen} onClose={() => setDebugOpen(false)} />
      </WizardProvider>
    </I18nProvider>
  )
}
