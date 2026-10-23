/**
 * React port of upstream `WizardContainer.qml`'s state: `<WizardProvider>`
 * owns it and `useWizard()` reads it together with the navigation API.
 */
import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useMemo,
  useRef,
  useState,
} from 'react'

import { lastCustomizationStep } from './navigation'
import { initialState, type WizardState } from './state'
import { TOTAL_STEPS } from './stepIds'
import { type Commit, useWizardActions, type WizardActions } from './useWizardActions'
import { useWizardBackend } from './useWizardBackend'

export interface WizardApi extends WizardState, WizardActions {
  /** False until the saved settings and first step have loaded. */
  ready: boolean
  totalSteps: number
  /** Upstream `getLastCustomizationStep()`. */
  lastCustomizationStep: number
}

const WizardContext = createContext<WizardApi | null>(null)

export interface WizardProviderProps {
  children: ReactNode
  /** Overrides the backend's `showLanguageSelection` answer (tests, CLI flag). */
  showLanguageSelection?: boolean
  /** Starting state overrides, for tests. */
  initial?: Partial<WizardState>
}

export function WizardProvider({ children, showLanguageSelection, initial }: WizardProviderProps) {
  const [state, setState] = useState<WizardState>(() => ({ ...initialState, ...initial }))
  const ref = useRef<WizardState>(state)
  const [ready, setReady] = useState(false)
  const commit: Commit = useCallback((change) => {
    ref.current = change(ref.current)
    setState(ref.current)
  }, [])
  const actions = useWizardActions(ref, commit)
  useWizardBackend(commit, setReady, showLanguageSelection)
  const value = useMemo<WizardApi>(
    () => ({
      ...state,
      ...actions,
      ready,
      totalSteps: TOTAL_STEPS,
      lastCustomizationStep: lastCustomizationStep(state),
    }),
    [state, actions, ready],
  )
  return <WizardContext.Provider value={value}>{children}</WizardContext.Provider>
}

/** The wizard state and navigation API; must be used inside `<WizardProvider>`. */
export function useWizard(): WizardApi {
  const value = useContext(WizardContext)
  if (!value) throw new Error('useWizard must be used inside <WizardProvider>')
  return value
}
