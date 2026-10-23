/**
 * Token state machine of upstream `PiConnectCustomizationStep.qml`: the
 * "Open Raspberry Pi Connect" button starts a 25 s countdown, after which
 * the token field unlocks for pasting; a token arriving from the browser
 * fills the field and keeps it read-only.
 */
export const COUNTDOWN_SECONDS = 25

export interface ConnectState {
  /** The "Enable Raspberry Pi Connect" switch. */
  enabled: boolean
  /** Text in the token field. */
  text: string
  /** Trimmed token (upstream `connectToken`). */
  token: string
  received: boolean
  fromBrowser: boolean
  fieldEnabled: boolean
  countdown: number
  running: boolean
}

export type ConnectAction =
  | { type: 'toggle'; value: boolean }
  | { type: 'start' }
  | { type: 'tick' }
  | { type: 'browserToken'; token: string; enable?: boolean }
  | { type: 'typed'; text: string }
  | { type: 'cleared' }
  | { type: 'invalidOk' }

export const initialConnectState: ConnectState = {
  enabled: false,
  text: '',
  token: '',
  received: false,
  fromBrowser: false,
  fieldEnabled: false,
  countdown: COUNTDOWN_SECONDS,
  running: false,
}

function tick(state: ConnectState): ConnectState {
  if (!state.running) return state
  const countdown = Math.max(0, state.countdown - 1)
  return countdown === 0
    ? { ...state, countdown, running: false, fieldEnabled: true }
    : { ...state, countdown }
}

function typed(state: ConnectState, text: string): ConnectState {
  const token = text.trim()
  return token
    ? { ...state, text, token, running: false }
    : { ...state, text, token: '', received: false }
}

export function connectReducer(state: ConnectState, action: ConnectAction): ConnectState {
  switch (action.type) {
    case 'toggle':
      return { ...state, enabled: action.value }
    case 'start':
      return state.received || state.running
        ? state
        : { ...state, countdown: COUNTDOWN_SECONDS, running: true }
    case 'tick':
      return tick(state)
    case 'browserToken':
      if (!action.token) return state
      return {
        ...state,
        enabled: action.enable ? true : state.enabled,
        text: action.token,
        token: action.token,
        received: true,
        fromBrowser: true,
        running: false,
        fieldEnabled: false,
      }
    case 'typed':
      return typed(state, action.text)
    case 'cleared':
      return initialConnectState
    case 'invalidOk':
      return { ...state, text: '', token: '', received: false }
  }
}

/** Upstream `nextButtonEnabled`. */
export const canProceed = (state: ConnectState): boolean =>
  !state.enabled || state.token.trim().length > 0
