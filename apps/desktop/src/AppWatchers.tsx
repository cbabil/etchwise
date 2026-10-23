/**
 * App-wide pieces of upstream's `main.qml` / `WizardContainer.qml` that do
 * not belong to one step: dialogs driven by backend signals, and the
 * storage-removal watch on the customisation steps (the Writing step runs
 * its own).
 */
import { BackendDialogs } from './wizard/dialogs/BackendDialogs'
import { Step } from './wizard/stepIds'
import { useStorageRemovalWatch } from './wizard/steps/storage/deviceRemoval'
import { useWizard } from './wizard/WizardContext'

export function AppWatchers() {
  const wizard = useWizard()
  const onCustomisation = wizard.currentStep > Step.Storage && wizard.currentStep < Step.Writing
  useStorageRemovalWatch(!wizard.isWriting && onCustomisation)
  return <BackendDialogs />
}
