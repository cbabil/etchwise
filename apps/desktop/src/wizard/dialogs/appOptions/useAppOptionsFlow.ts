/** Sub-dialog flow of upstream `AppOptionsDialog.qml`: confirmations, key picker, repository. */
import { useState } from 'react'

import { call } from '@/api/imager'
import { useTr } from '@/i18n'

import { useWizard } from '../../WizardContext'
import { pickFile } from '../common/pickFile'
import { type AppOptionsValues, useAppOptions } from './useAppOptions'

type Change = (patch: Partial<AppOptionsValues>) => void

/** "Disable warnings" switch: turning it on asks first; Cancel / Escape turns it back off. */
function useDisableWarnings(change: Change) {
  const wizard = useWizard()
  const [open, setOpen] = useState(false)
  const toggle = (checked: boolean) => {
    change({ disableWarnings: checked })
    if (checked) setOpen(true)
    else wizard.update({ disableWarnings: false })
  }
  const cancel = () => {
    setOpen(false)
    change({ disableWarnings: false })
  }
  const confirm = () => {
    setOpen(false)
    wizard.update({ disableWarnings: true })
  }
  return { open, toggle, cancel, confirm }
}

/** "Saved Customisation - Clear" with its confirmation. */
function useClearSettings(onClose: () => void) {
  const wizard = useWizard()
  const [open, setOpen] = useState(false)
  const confirm = () => {
    void call('clearSavedCustomisationSettings')
    wizard.setCustomizationSettings({})
    wizard.update({
      hostnameConfigured: false,
      localeConfigured: false,
      userConfigured: false,
      wifiConfigured: false,
      sshEnabled: false,
    })
    setOpen(false)
    onClose()
  }
  return { open, start: () => setOpen(true), cancel: () => setOpen(false), confirm }
}

export function useAppOptionsFlow(open: boolean, onClose: () => void) {
  const tr = useTr('AppOptionsDialog')
  const options = useAppOptions(open)
  const warnings = useDisableWarnings(options.change)
  const clear = useClearSettings(onClose)
  const [repoOpen, setRepoOpen] = useState(false)
  const selectKey = async () => {
    const path = await pickFile({
      title: tr('Select RSA Private Key'),
      filters: [
        { name: tr('PEM Files (*.pem)'), extensions: ['pem'] },
        { name: tr('All Files (*)'), extensions: ['*'] },
      ],
    })
    if (path) options.change({ rsaKeyPath: path })
  }
  const editRepository = () => {
    onClose()
    setRepoOpen(true)
  }
  return {
    options,
    warnings,
    clear,
    repoOpen,
    selectKey,
    editRepository,
    closeRepo: () => setRepoOpen(false),
  }
}
