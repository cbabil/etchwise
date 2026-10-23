/**
 * Upstream `DeviceSelectionStep.qml`: the Raspberry Pi device list. A
 * click (or Enter / Space) selects the device on the backend and names it
 * in the wizard; keyboard selection and double-click also advance. Next is
 * enabled once a device is highlighted, or while offline with no list so a
 * local image can still be written.
 */
import { Box } from '@mui/material'

import { call } from '@/api/imager'
import { SelectionList } from '@/components/SelectionList'
import { useTr } from '@/i18n'
import { colors } from '@/theme/tokens'

import { StepBase } from '../StepBase'
import { useWizard } from '../WizardContext'
import { DeviceOfflinePlaceholder } from './device/DeviceOfflinePlaceholder'
import { useHwList } from './device/useHwList'
import { ListRow, MetaLine } from './os/ListRow'
import type { HwDevice } from './os/types'

const ROW_MIN_HEIGHT = 60

function useListName(count: number): string {
  const tr = useTr('DeviceSelectionStep')
  const amount =
    count === 0 ? tr('No devices') : count === 1 ? tr('1 device') : tr('%1 devices', count)
  return `${tr('Device selection list')}. ${amount}. ${tr('Use arrow keys to navigate, Enter or Space to select')}`
}

function DeviceRow({ device, current }: { device: HwDevice; current: boolean }) {
  const tr = useTr('DeviceSelectionStep')
  return (
    <ListRow
      current={current}
      minHeight={ROW_MIN_HEIGHT}
      icon={device.icon}
      name={device.name}
      description={device.description}
    >
      {device.isUsbBootConnected ? (
        <MetaLine color={colors.formControlActive}>{tr('Connected via USB')}</MetaLine>
      ) : null}
    </ListRow>
  )
}

export function DeviceSelectionStep() {
  const tr = useTr('DeviceSelectionStep')
  const wizard = useWizard()
  const { devices, current, setCurrent, unavailable } = useHwList()
  const listName = useListName(devices.length)
  const nextEnabled = current !== -1 || (unavailable && devices.length === 0)
  const choose = (index: number, device: HwDevice) => {
    setCurrent(index)
    void call('setHWIndex', index)
    wizard.setSelectedDeviceName(device.name)
  }
  return (
    <StepBase
      title={tr('Select your Raspberry Pi device')}
      showBack={false}
      nextEnabled={nextEnabled}
      onNext={wizard.next}
    >
      {devices.length === 0 && unavailable ? (
        <DeviceOfflinePlaceholder />
      ) : (
        <Box sx={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
          <SelectionList
            items={devices}
            getKey={(device, index) => `${index}:${device.name}`}
            currentIndex={current}
            onCurrentIndexChange={setCurrent}
            onItemSelected={choose}
            onItemDoubleClicked={(index, device) => {
              choose(index, device)
              queueMicrotask(wizard.next)
            }}
            keyboardAutoAdvance
            onNext={wizard.next}
            ariaLabel={listName}
            renderItem={(device, index, isCurrent) => (
              <DeviceRow key={index} device={device} current={isCurrent} />
            )}
            sx={{ flex: 1 }}
          />
        </Box>
      )}
    </StepBase>
  )
}
