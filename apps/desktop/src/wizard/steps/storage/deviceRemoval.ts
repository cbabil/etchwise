/**
 * Selected-device removal (upstream `ImageWriter::onSelectedDeviceRemoved`
 * plus `main.qml`'s `onSelectedDeviceRemoved` and
 * `onWriteCancelledDueToDeviceRemoval`).
 */
import { useCallback } from 'react'

import { Step } from '../../stepIds'
import { useWizard } from '../../WizardContext'
import type { DriveList } from './drives'
import { selectedDrive, setDst, setRemovedNotice } from './selectedDrive'
import { usePollDrives } from './useDriveList'

/** True when a storage device is selected but missing from a good list. */
export function isSelectedRemoved(list: DriveList, storageName: string): boolean {
  const selected = selectedDrive()
  if (!storageName || !selected || list.error) return false
  return !list.drives.some((d) => d.device === selected.device)
}

/**
 * Clears the selection; past the Storage step it also jumps back there
 * and shows the "Storage device removed" dialog.
 */
export function useClearRemovedStorage(): (navigate: boolean) => void {
  const { setSelectedStorageName, jumpTo } = useWizard()
  return useCallback(
    (navigate: boolean) => {
      setSelectedStorageName('')
      setDst('', 0)
      if (!navigate) return
      jumpTo(Step.Storage)
      setRemovedNotice(true)
    },
    [setSelectedStorageName, jumpTo],
  )
}

/**
 * Watches the drive list while `enabled` (a step past Storage with no
 * write running: upstream pauses polling during writes) and returns to
 * the Storage step when the selected device disappears.
 */
export function useStorageRemovalWatch(enabled: boolean): void {
  const { selectedStorageName, currentStep } = useWizard()
  const clear = useClearRemovedStorage()
  usePollDrives((list) => {
    if (isSelectedRemoved(list, selectedStorageName)) clear(currentStep > Step.Storage)
  }, enabled)
}
