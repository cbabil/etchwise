/**
 * Upstream `StorageSelectionStep.qml`: the polled storage device list,
 * the "Exclude system drives" filter, and the two stern confirmations.
 * Next stays disabled until a device is selected.
 */
import { Box } from '@mui/material'

import { ImCheckBox } from '@/components/ImCheckBox'
import { SelectionList } from '@/components/SelectionList'
import { useTr } from '@/i18n'

import { ConfirmSystemDriveDialog } from '../dialogs/ConfirmSystemDriveDialog'
import { ConfirmUnfilterDialog } from '../dialogs/ConfirmUnfilterDialog'
import { StepBase } from '../StepBase'
import { useWizard } from '../WizardContext'
import { listAccessibleName, storageStatus, storageStatusMessage } from './storage/drives'
import { StorageRemovedDialog } from './storage/StorageRemovedDialog'
import { StorageRow } from './storage/StorageRow'
import { EnumerationErrorBanner, NoDevicesStatus } from './storage/StorageStatus'
import { useStorageSelection } from './storage/useStorageSelection'

type Selection = ReturnType<typeof useStorageSelection>

function DriveList({ s }: { s: Selection }) {
  const tr = useTr('StorageSelectionStep')
  const status = storageStatus(s.list, s.filter)
  const selectable = s.visible.filter((d) => !d.isReadOnly).length
  const message = status === 'ok' ? '' : storageStatusMessage(status, s.list.error, tr)
  return (
    <Box sx={{ position: 'relative', flex: 1, minHeight: 0, display: 'flex' }}>
      <SelectionList
        items={s.visible}
        getKey={(d) => d.key}
        currentIndex={s.currentIndex}
        onCurrentIndexChange={s.setCurrentIndex}
        isItemSelectable={(i) => s.visible[i]?.isReadOnly === false}
        onItemSelected={(_, d) => s.selectDrive(d)}
        onItemDoubleClicked={(_, d) => s.onDoubleClick(d)}
        keyboardAutoAdvance
        onNext={s.advanceAfterKey}
        ariaLabel={listAccessibleName(selectable, message, tr)}
        renderItem={(d, _, current) => <StorageRow drive={d} current={current} />}
        sx={{ flex: 1 }}
      />
      <NoDevicesStatus status={status} error={s.list.error} />
    </Box>
  )
}

export function StorageSelectionStep() {
  const tr = useTr('StorageSelectionStep')
  const wizard = useWizard()
  const s = useStorageSelection()
  return (
    <StepBase
      title={tr('Select your storage device')}
      nextEnabled={wizard.selectedStorageName.length > 0}
      onNext={wizard.next}
      onBack={wizard.back}
    >
      <EnumerationErrorBanner error={s.list.error} />
      <DriveList s={s} />
      <Box sx={{ display: 'flex', justifyContent: 'flex-end', flexShrink: 0 }}>
        <ImCheckBox
          label={tr('Exclude system drives')}
          checked={s.filter}
          onChange={s.onFilterChange}
          accessibleDescription={tr(
            'When checked, system drives are hidden from the list. Uncheck to show all drives including system drives.',
          )}
        />
      </Box>
      <ConfirmUnfilterDialog
        open={s.unfilterOpen}
        onConfirm={s.confirmUnfilter}
        onCancel={s.cancelUnfilter}
      />
      <ConfirmSystemDriveDialog {...s.systemDialog} />
      <StorageRemovedDialog />
    </StepBase>
  )
}
