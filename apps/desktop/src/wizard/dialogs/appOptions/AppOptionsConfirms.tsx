/** "Disable warnings?" and "Clear saved customisation?" confirmations of `AppOptionsDialog.qml`. */
import { useTr } from '@/i18n'

import { ConfirmDialog, styledText } from '../common/ConfirmDialog'

interface ConfirmProps {
  open: boolean
  onCancel: () => void
  onConfirm: () => void
}

export function ConfirmDisableWarnings({ open, onCancel, onConfirm }: ConfirmProps) {
  const tr = useTr('AppOptionsDialog')
  return (
    <ConfirmDialog
      open={open}
      title={tr('Disable warnings?')}
      body={styledText(
        tr(
          'If you disable warnings, Raspberry Pi Imager will <b>not show confirmation prompts before writing images</b>. You will still be required to <b>type the exact name</b> when selecting a system drive.',
        ),
      )}
      cancelDescription={tr('Keep warnings enabled and return to the options dialog')}
      confirmLabel={tr('Disable warnings')}
      confirmDescription={tr(
        'Disable confirmation prompts before writing images, requiring only exact name entry for system drives',
      )}
      onCancel={onCancel}
      onConfirm={onConfirm}
    />
  )
}

export function ConfirmClearSettings({ open, onCancel, onConfirm }: ConfirmProps) {
  const tr = useTr('AppOptionsDialog')
  return (
    <ConfirmDialog
      open={open}
      title={tr('Clear saved customisation?')}
      body={tr(
        'This will remove all saved OS customisation settings such as hostname, WiFi, and user credentials.',
      )}
      cancelDescription={tr('Keep saved customisation settings and return to the options dialog')}
      confirmLabel={tr('Clear')}
      confirmDescription={tr('Remove all saved OS customisation settings permanently')}
      onCancel={onCancel}
      onConfirm={onConfirm}
    />
  )
}
