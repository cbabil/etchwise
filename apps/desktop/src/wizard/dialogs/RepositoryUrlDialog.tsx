/**
 * `WizardContainer.qml` deep-link repository dialog: opens on the backend
 * `repositoryUrlReceived(url)` signal. Remote URLs unlock "Switch
 * repository" after 1.5 s; local `file://` manifests may be opened at
 * once. A second URL is ignored while the dialog is showing another one.
 */
import { Box } from '@mui/material'
import { useState } from 'react'

import { call } from '@/api/imager'
import { BaseDialog } from '@/components/BaseDialog'
import { DialogButtons, DialogHeading } from '@/components/DialogText'
import { ImButton } from '@/components/ImButton'
import { ImButtonRed } from '@/components/ImButtonRed'
import { useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

import { useWizard } from '../WizardContext'
import { InfoText } from './common/dialogParts'
import { useDelayedFlag } from './common/useDelayedFlag'
import { useImagerEvent } from './common/useImagerEvent'

export const REPO_ACCEPT_DELAY_MS = 1500

function UrlBox({ url }: { url: string }) {
  const tr = useTr('WizardContainer')
  return (
    <Box
      aria-label={tr('Repository URL: %1', url)}
      sx={{
        p: `${spacing.small}px`,
        backgroundColor: colors.titleBackground,
        border: `1px solid ${colors.popupBorder}`,
        borderRadius: `${sizes.listItemBorderRadius}px`,
        fontFamily: 'monospace',
        fontSize: fontSize.xs,
        color: colors.formLabel,
        overflowWrap: 'anywhere',
        display: '-webkit-box',
        WebkitLineClamp: 3,
        WebkitBoxOrient: 'vertical',
        overflow: 'hidden',
      }}
    >
      {url}
    </Box>
  )
}

function useRepoUrl() {
  const [url, setUrl] = useState<string | null>(null)
  useImagerEvent('repositoryUrlReceived', (data) => {
    const next = String(data ?? '')
    setUrl((current) => (current !== null && current !== next ? current : next))
  })
  const isLocal = url?.startsWith('file://') ?? false
  const delayed = useDelayedFlag(url !== null && !isLocal, REPO_ACCEPT_DELAY_MS)
  return { url, setUrl, isLocal, allowAccept: url !== null && (isLocal || delayed) }
}

function useRepoTexts(isLocal: boolean, allowAccept: boolean) {
  const tr = useTr('WizardContainer')
  return {
    title: isLocal ? tr('Open local repository file?') : tr('Switch to a custom repository?'),
    body: isLocal
      ? tr(
          'You are opening a local Raspberry Pi Imager manifest file. This will replace the current OS list with the contents of this file.',
        )
      : tr(
          'A website is requesting to switch Raspberry Pi Imager to use a custom OS repository.\n\n',
        ) +
        tr(
          'Only accept if you trust this source and intentionally clicked a link to open this repository.',
        ),
    accept: !allowAccept ? tr('Please wait\u2026') : isLocal ? tr('Open') : tr('Switch repository'),
    acceptDescription: isLocal
      ? tr('Open the local manifest file and use it as the OS repository')
      : tr('Switch to the custom repository from the link'),
  }
}

export function RepositoryUrlDialog() {
  const tr = useTr('WizardContainer')
  const wizard = useWizard()
  const { url, setUrl, isLocal, allowAccept } = useRepoUrl()
  const text = useRepoTexts(isLocal, allowAccept)
  const close = () => setUrl(null)
  const accept = () => {
    if (url) void call('refreshOsListFrom', url)
    close()
    wizard.resetWizard()
  }
  return (
    <BaseDialog open={url !== null} onClose={close} title={text.title}>
      <DialogHeading>{text.title}</DialogHeading>
      <InfoText size={fontSize.sm} color={colors.formLabel}>
        {text.body}
      </InfoText>
      <UrlBox url={url ?? ''} />
      <DialogButtons>
        <ImButton
          onClick={accept}
          disabled={!allowAccept}
          accessibleDescription={text.acceptDescription}
        >
          {text.accept}
        </ImButton>
        <ImButtonRed
          onClick={close}
          accessibleDescription={tr('Keep your current repository settings')}
        >
          {tr('Cancel')}
        </ImButtonRed>
      </DialogButtons>
    </BaseDialog>
  )
}
