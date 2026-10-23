/** Backend wiring and Next handling of upstream `PiConnectCustomizationStep.qml`. */
import { type Dispatch, useEffect, useReducer, useState } from 'react'

import { call } from '@/api/imager'

import { useImagerEvent } from '../../dialogs/common/useImagerEvent'
import { useWizard } from '../../WizardContext'
import { type ConnectAction, connectReducer, initialConnectState } from './piConnectState'

export const CONNECT_AUTH_URL = 'https://connect.raspberrypi.com/imager/'

function useTokenSources(dispatch: Dispatch<ConnectAction>) {
  const wizard = useWizard()
  const { update, removeCustomizationSetting } = wizard
  useEffect(() => {
    let live = true
    void call<string | null>('getRuntimeConnectToken').then((token) => {
      if (!live || !token) return
      dispatch({ type: 'browserToken', token, enable: true })
      update({ piConnectEnabled: true })
    })
    return () => {
      live = false
    }
  }, [dispatch, update])
  useImagerEvent('connectTokenReceived', (data) => {
    dispatch({ type: 'browserToken', token: typeof data === 'string' ? data : '' })
  })
  useImagerEvent('connectTokenCleared', () => {
    dispatch({ type: 'cleared' })
    update({ piConnectEnabled: false })
    removeCustomizationSetting('piConnectEnabled')
  })
}

export function usePiConnect() {
  const wizard = useWizard()
  const [state, dispatch] = useReducer(connectReducer, initialConnectState)
  const [invalidOpen, setInvalidOpen] = useState(false)
  useTokenSources(dispatch)
  useEffect(() => {
    if (!state.running) return
    const timer = setInterval(() => dispatch({ type: 'tick' }), 1000)
    return () => clearInterval(timer)
  }, [state.running])
  const toggle = (value: boolean) => {
    dispatch({ type: 'toggle', value })
    wizard.update({ piConnectEnabled: value })
  }
  const openConnect = () => {
    void call('openUrl', CONNECT_AUTH_URL)
    dispatch({ type: 'start' })
  }
  const next = async () => {
    if (!state.enabled) {
      wizard.removeCustomizationSetting('piConnectEnabled')
      wizard.update({ piConnectEnabled: false })
      return wizard.next()
    }
    if (!state.fromBrowser) {
      const token = state.token.trim()
      const valid = token ? await call<boolean | null>('verifyAuthKey', token, true) : false
      if (!valid) return setInvalidOpen(true)
      await call('overwriteConnectToken', token)
    }
    wizard.updateCustomizationSettings({ piConnectEnabled: true })
    wizard.update({ piConnectEnabled: true })
    wizard.next()
  }
  const invalidOk = () => {
    setInvalidOpen(false)
    dispatch({ type: 'invalidOk' })
  }
  const dismissInvalid = () => setInvalidOpen(false)
  return { state, dispatch, toggle, openConnect, next, invalidOpen, invalidOk, dismissInvalid }
}
