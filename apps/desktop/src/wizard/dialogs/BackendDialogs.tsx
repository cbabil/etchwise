/**
 * The dialogs upstream `WizardContainer.qml` opens on backend signals:
 * the Raspberry Pi Connect token conflict and the deep-link custom
 * repository. Mount once inside `<WizardProvider>`.
 */
import { RepositoryUrlDialog } from './RepositoryUrlDialog'
import { TokenConflictDialog } from './TokenConflictDialog'

export function BackendDialogs() {
  return (
    <>
      <TokenConflictDialog />
      <RepositoryUrlDialog />
    </>
  )
}
