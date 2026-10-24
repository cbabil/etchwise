/**
 * The extra repository of App Options (not in upstream): an OS list whose
 * devices and images are merged on top of the main one, kept in the
 * `extraRepo` setting. Saving a changed choice reloads the lists and
 * restarts the wizard, as the Content Repository dialog does.
 */
import { useEffect, useState } from 'react'

import { call } from '@/api/imager'
import { useTr } from '@/i18n'

import { useWizard } from '../../WizardContext'
import { pickFile } from '../common/pickFile'
import { fileUrlOf } from '../repository/useRepository'

export const EXTRA_REPO_SETTING = 'extraRepo'

export interface ExtraRepo {
  url: string
  /** Why the saved extra repository did not load; empty when it did. */
  error: string
  choose: () => Promise<void>
  remove: () => void
  save: () => void
}

export function useExtraRepo(open: boolean): ExtraRepo {
  const common = useTr('CommonStrings')
  const wizard = useWizard()
  const [saved, setSaved] = useState('')
  const [url, setUrl] = useState('')
  const [error, setError] = useState('')
  useEffect(() => {
    if (!open) return
    let live = true
    void Promise.all([
      call<string | null>('getStringSetting', EXTRA_REPO_SETTING),
      call<string | null>('getExtraRepoError'),
    ]).then(([current, failure]) => {
      if (!live) return
      setSaved(current ?? '')
      setUrl(current ?? '')
      setError(failure ?? '')
    })
    return () => {
      live = false
    }
  }, [open])
  const choose = async () => {
    const path = await pickFile({
      title: 'Select Extra Repository',
      filters: [
        {
          name: common('Imager Repository Files (*.json *.%1)', 'rpi-imager-manifest'),
          extensions: ['json', 'rpi-imager-manifest'],
        },
      ],
    })
    if (path) {
      setUrl(fileUrlOf(path))
      setError('')
    }
  }
  const remove = () => {
    setUrl('')
    setError('')
  }
  const save = () => {
    if (url === saved) return
    void call('setSetting', EXTRA_REPO_SETTING, url).then(() => call('beginOSListFetch'))
    wizard.resetWizard()
  }
  return { url, error, choose, remove, save }
}
