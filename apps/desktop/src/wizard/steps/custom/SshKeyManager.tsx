/**
 * Upstream `wizard/components/SshKeyManager.qml`: "N SSH keys configured"
 * with SHOW / HIDE; expanded, one row per key (shortened monospace text
 * and REMOVE) and a paste field whose button reads ADD when it has text
 * and BROWSE otherwise. Keys are trimmed and de-duplicated.
 */
import { Box } from '@mui/material'
import { type KeyboardEvent, useState } from 'react'

import { ImButton } from '@/components/ImButton'
import { ImTextField } from '@/components/ImTextField'
import { useTr } from '@/i18n'
import { colors, fontSize, spacing } from '@/theme/tokens'

import { browseKeyFile } from './browseKeyFile'
import { FormRow } from './layout'
import { SshKeyRow } from './SshKeyRow'
import { addKey, dedupeKeys, splitKeys } from './sshKeys'

export interface SshKeyManagerProps {
  keys: readonly string[]
  onChange: (keys: readonly string[]) => void
  disabled?: boolean
}

// Upstream: at least 80px, and never narrower than its label beside the
// key field, which takes the rest of the row
const small = { minWidth: 80, flexShrink: 0 } as const

function AddKeyRow({ keys, onChange, disabled }: SshKeyManagerProps) {
  const tr = useTr('SshKeyManager')
  const common = useTr('CommonStrings')
  const [text, setText] = useState('')
  const hasText = text.trim().length > 0
  const add = () => {
    onChange(addKey(keys, text))
    setText('')
  }
  const browse = async () => {
    const contents = await browseKeyFile(tr, common)
    if (contents) onChange(dedupeKeys([...keys, ...splitKeys(contents)]))
  }
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Enter' && hasText) add()
  }
  return (
    <FormRow>
      <ImTextField
        fullWidth
        disabled={disabled ?? false}
        placeholder={tr('Paste key or click BROWSE to select file')}
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={onKeyDown}
        slotProps={{
          htmlInput: {
            'aria-label': tr('SSH public key input'),
            'aria-description': tr(
              'Paste an SSH public key here or use the browse button to select a key file',
            ),
          },
        }}
      />
      <ImButton
        sx={small}
        disabled={disabled ?? false}
        accessibleDescription={
          hasText ? tr('Add the entered SSH key') : tr('Select an SSH public key file to add')
        }
        onClick={() => (hasText ? add() : void browse())}
      >
        {hasText ? tr('Add') : common('Browse')}
      </ImButton>
    </FormRow>
  )
}

export function SshKeyManager(props: SshKeyManagerProps) {
  const tr = useTr('SshKeyManager')
  const [expanded, setExpanded] = useState(false)
  const { keys, onChange, disabled = false } = props
  const summary =
    keys.length === 0
      ? tr('No SSH keys configured')
      : keys.length === 1
        ? tr('1 SSH key configured')
        : tr('%1 SSH keys configured', keys.length)
  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.small}px` }}>
      <FormRow>
        <Box sx={{ flex: 1, fontSize: fontSize.sm, color: colors.formLabel }}>{summary}</Box>
        <ImButton
          sx={small}
          disabled={disabled}
          accessibleDescription={
            expanded ? tr('Hide the list of SSH keys') : tr('Show the list of SSH keys')
          }
          onClick={() => setExpanded((e) => !e)}
        >
          {expanded ? tr('Hide') : tr('Show')}
        </ImButton>
      </FormRow>
      {expanded ? (
        <Box
          role="group"
          aria-label={tr('SSH keys list')}
          sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.small}px` }}
        >
          {keys.map((key, index) => (
            <SshKeyRow
              key={key}
              sshKey={key}
              index={index}
              disabled={disabled}
              onRemove={() => onChange(keys.filter((_, i) => i !== index))}
            />
          ))}
          <AddKeyRow {...props} />
        </Box>
      ) : null}
    </Box>
  )
}
