/** Loading, auto-skip and Next handling of upstream `IfAndFeaturesCustomizationStep.qml`. */
import { useEffect, useRef, useState } from 'react'

import { call } from '@/api/imager'

import { useWizard } from '../../WizardContext'
import {
  hasAnyCap,
  type IfCaps,
  type IfValues,
  initialValues,
  loadCaps,
  noIfFeatures,
  resultOf,
} from './ifFeatures'

const PERSISTED_KEYS = ['enableI2C', 'enableSPI', 'enable1Wire', 'enableSerial', 'enableUsbGadget']

function useCapsLoader(onLoaded: (caps: IfCaps, values: IfValues) => void) {
  const wizard = useWizard()
  const initial = useRef({ wizard, onLoaded, skipped: false })
  useEffect(() => {
    const { wizard: w } = initial.current
    if (!w.ccRpiAvailable) {
      if (initial.current.skipped) return
      initial.current.skipped = true
      w.update({ ...noIfFeatures, ifAndFeaturesAvailable: false, ifSerial: '' })
      w.next()
      return
    }
    let live = true
    void loadCaps().then((caps) => {
      if (!live) return
      const any = hasAnyCap(caps)
      w.update({ ifAndFeaturesAvailable: any })
      if (!any) {
        w.update({ ...noIfFeatures, ifSerial: 'Disabled' })
        w.next()
        return
      }
      initial.current.onLoaded(caps, initialValues(caps, w.customizationSettings, w))
    })
    return () => {
      live = false
    }
  }, [])
}

export function useIfFeatures() {
  const wizard = useWizard()
  const [caps, setCaps] = useState<IfCaps | null>(null)
  const [values, setValues] = useState<IfValues | null>(null)
  const [confirmOpen, setConfirmOpen] = useState(false)
  useCapsLoader((loaded, prefill) => {
    setCaps(loaded)
    setValues(prefill)
  })
  const change = (patch: Partial<IfValues>) => setValues((v) => (v ? { ...v, ...patch } : v))
  const next = () => {
    if (!caps || !values) return
    const result = resultOf(caps, values)
    wizard.updateCustomizationSettings(result.settings)
    PERSISTED_KEYS.forEach((key) => void call('removePersistedCustomisationSetting', key))
    wizard.update(result.flags)
    if (result.settings.enableUsbGadget && !wizard.disableWarnings) setConfirmOpen(true)
    else wizard.next()
  }
  const accept = () => {
    setConfirmOpen(false)
    wizard.next()
  }
  return { caps, values, change, next, confirmOpen, accept, cancel: () => setConfirmOpen(false) }
}
