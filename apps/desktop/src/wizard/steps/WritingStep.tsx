/**
 * Upstream `WritingStep.qml`: the summary of choices and customisations,
 * the ERASE confirmation, then progress while writing and verifying.
 * Back is disabled while a write runs; success advances to Done.
 */
import { Box } from '@mui/material'
import { useEffect, useState } from 'react'

import { call } from '@/api/imager'
import { useTr } from '@/i18n'
import { sizes, spacing } from '@/theme/tokens'

import { StepBase } from '../StepBase'
import { useWizard } from '../WizardContext'
import { useStorageRemovalWatch } from './storage/deviceRemoval'
import { EraseConfirmDialog } from './writing/EraseConfirmDialog'
import { ProgressSection } from './writing/ProgressSection'
import { useWriteJob } from './writing/useWriteJob'
import { phaseOf, stepTexts } from './writing/writeButtons'
import { WriteErrorDialog } from './writing/WriteErrorDialog'
import { WriteSummary } from './writing/WriteSummary'

/** `imageWriter.readyToWrite()`, re-asked when the storage selection changes. */
function useReadyToWrite(storageName: string): boolean {
  const [ready, setReady] = useState(false)
  useEffect(() => {
    let live = true
    void call<boolean | null>('readyToWrite').then((value) => {
      if (live) setReady(Boolean(value))
    })
    return () => {
      live = false
    }
  }, [storageName])
  return ready
}

function Content({
  job,
  showProgress,
}: {
  job: ReturnType<typeof useWriteJob>
  showProgress: boolean
}) {
  return (
    <Box
      sx={{
        flex: 1,
        minHeight: 0,
        display: 'flex',
        flexDirection: 'column',
        justifyContent: showProgress ? 'center' : 'flex-start',
        gap: `${spacing.large}px`,
        p: `${sizes.cardPadding}px`,
      }}
    >
      {showProgress ? <ProgressSection job={job} /> : <WriteSummary />}
    </Box>
  )
}

export function WritingStep() {
  const tr = useTr('WritingStep')
  const common = useTr('CommonStrings')
  const wizard = useWizard()
  const job = useWriteJob()
  const [confirmOpen, setConfirmOpen] = useState(false)
  const ready = useReadyToWrite(wizard.selectedStorageName)
  useStorageRemovalWatch(job.progress.state === 'Idle')
  const phase = phaseOf(job.progress.state, job.active)
  const texts = stepTexts(phase, tr, common)
  const onNext = () => {
    if (phase === 'verifying') job.skipVerification()
    else if (phase === 'writing') job.cancel()
    else if (phase === 'complete') wizard.next()
    else if (wizard.disableWarnings) job.delay.begin()
    else setConfirmOpen(true)
  }
  return (
    <StepBase
      title={tr('Write image')}
      subtitle={texts.subtitle}
      nextLabel={texts.next}
      nextAccessibleDescription={texts.nextDescription}
      backAccessibleDescription={tr('Return to previous customization step')}
      nextEnabled={phase !== 'review' || (!job.delay.running && ready)}
      backEnabled={!job.active}
      onNext={onNext}
      onBack={wizard.back}
    >
      <Content job={job} showProgress={phase !== 'review'} />
      <EraseConfirmDialog
        open={confirmOpen}
        storageName={wizard.selectedStorageName}
        onCancel={() => setConfirmOpen(false)}
        onAccept={() => {
          setConfirmOpen(false)
          job.delay.begin()
        }}
      />
      <WriteErrorDialog message={job.extras.startError} onClose={() => job.clearStartError()} />
    </StepBase>
  )
}
