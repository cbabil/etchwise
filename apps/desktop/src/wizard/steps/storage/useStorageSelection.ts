/**
 * Behaviour of upstream `StorageSelectionStep.qml`: the polled list, the
 * "Exclude system drives" filter with its stern confirmation, selecting a
 * drive (`selectDstItem`), the typed-name confirmation for system drives,
 * and clearing the selection when the selected device disappears.
 */
import { useRef, useState } from 'react'

import { call } from '@/api/imager'
import { formatSize } from '@/format'

import { useWizard } from '../../WizardContext'
import { isSelectedRemoved, useClearRemovedStorage } from './deviceRemoval'
import { type DriveInfo, driveName, unselectable, visibleDrives } from './drives'
import { setDst } from './selectedDrive'
import { useDriveList } from './useDriveList'

function useFilter(disableWarnings: boolean) {
  const [filter, setFilter] = useState(true)
  const [unfilterOpen, setUnfilterOpen] = useState(false)
  return {
    filter,
    setFilter,
    unfilterOpen,
    onFilterChange: (checked: boolean) => {
      setFilter(checked)
      if (!checked && !disableWarnings) setUnfilterOpen(true)
    },
    confirmUnfilter: () => setUnfilterOpen(false),
    cancelUnfilter: () => {
      setFilter(true)
      setUnfilterOpen(false)
    },
  }
}

function useSystemConfirm(choose: (d: DriveInfo) => void, next: () => void, refilter: () => void) {
  const [pending, setPending] = useState<DriveInfo | null>(null)
  return {
    pending,
    ask: setPending,
    systemDialog: {
      open: pending !== null,
      driveName: pending?.description ?? '',
      sizeStr: formatSize(pending?.size ?? 0),
      mountpoints: pending?.mountpoints ?? [],
      onCancel: () => setPending(null),
      onConfirm: () => {
        if (pending) choose(pending)
        setPending(null)
        refilter()
        next()
      },
    },
  }
}

export function useStorageSelection() {
  const wizard = useWizard()
  const clearRemoved = useClearRemovedStorage()
  const [currentKey, setCurrentKey] = useState<string | null>(null)
  const selected = useRef(false)
  const list = useDriveList((next) => {
    if (!isSelectedRemoved(next, wizard.selectedStorageName)) return
    clearRemoved(false)
    setCurrentKey(null)
  })
  const filters = useFilter(wizard.disableWarnings)
  const visible = visibleDrives(list.drives, filters.filter)
  const choose = (d: DriveInfo) => {
    if (d.isRpiboot) void call('setRpibootDevice', d.device)
    else setDst(d.device, d.size)
    wizard.setSelectedStorageName(driveName(d))
  }
  const confirm = useSystemConfirm(choose, wizard.next, () => filters.setFilter(true))
  const selectDrive = (d: DriveInfo): boolean => {
    selected.current = false
    if (unselectable(d)) return false
    if (d.isSystem) confirm.ask(d)
    else choose(d)
    selected.current = !d.isSystem
    return selected.current
  }
  return {
    list,
    visible,
    ...filters,
    systemDialog: confirm.systemDialog,
    currentIndex: visible.findIndex((d) => d.key === currentKey),
    setCurrentIndex: (index: number) => setCurrentKey(visible[index]?.key ?? null),
    selectDrive,
    /** Keyboard auto-advance (`conditionalNext`): never for system drives. */
    advanceAfterKey: () => {
      if (selected.current) wizard.next()
    },
    onDoubleClick: (d: DriveInfo) => {
      if (selectDrive(d)) wizard.next()
    },
  }
}
