/**
 * Upstream `OSSelectionStep.qml`: the operating system list with
 * drill-down groups, the offline banner, the update-available prompt and
 * the repository's default OS. Next is enabled once the main list has a
 * current row and an OS is named, or after a selection decided it.
 */
import { Box } from '@mui/material'
import { useEffect, useRef } from 'react'

import { useTr } from '@/i18n'

import { UpdateAvailableDialog } from '../dialogs/UpdateAvailableDialog'
import { StepBase } from '../StepBase'
import { useWizard } from '../WizardContext'
import { OsListView } from './os/OsListView'
import { OsOfflineBanner } from './os/OsOfflineBanner'
import { BACK_ROW, type OsEntry } from './os/types'
import { useCachedHashes } from './os/useCachedHashes'
import { useListMetadata } from './os/useListMetadata'
import { useOsList } from './os/useOsList'
import { useOsSelection } from './os/useOsSelection'

function useOsStep() {
  const wizard = useWizard()
  const handleRef = useRef<(entry: OsEntry, fromKeyboard: boolean) => Promise<void>>(
    async () => undefined,
  )
  const metadata = useListMetadata((entry) => void handleRef.current(entry, false))
  const list = useOsList((entries) => void metadata.onLoaded(entries))
  const selection = useOsSelection(wizard, list)
  useEffect(() => {
    handleRef.current = selection.handle
  })
  const cached = useCachedHashes(list.level.entries)
  const mainCurrent = list.levels[0]?.current ?? -1
  const nextEnabled =
    selection.nextOverride ?? (mainCurrent !== -1 && wizard.selectedOsName.length > 0)
  return { wizard, metadata, list, selection, cached, nextEnabled }
}

export function OSSelectionStep() {
  const tr = useTr('OSSelectionStep')
  const { wizard, metadata, list, selection, cached, nextEnabled } = useOsStep()
  return (
    <StepBase
      title={tr('Choose operating system')}
      subtitle={tr('Select an operating system to install on your Raspberry Pi')}
      showBack={wizard.hasNetworkConnectivity}
      nextEnabled={nextEnabled}
      onNext={wizard.next}
      onBack={wizard.back}
    >
      <Box sx={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
        {list.unavailable ? <OsOfflineBanner /> : null}
        <OsListView
          level={list.level}
          depth={list.levels.length - 1}
          ctx={{ cached, customSize: selection.customSize }}
          onCurrentChange={list.setCurrent}
          onSelect={(entry, fromKeyboard) => void selection.handle(entry, fromKeyboard)}
          onBack={() => void selection.handle(BACK_ROW, false)}
        />
      </Box>
      <UpdateAvailableDialog
        open={metadata.offer !== null}
        url={metadata.offer?.url ?? ''}
        version={metadata.offer?.version ?? ''}
        onClose={metadata.closeOffer}
      />
    </StepBase>
  )
}
