/** Source radio buttons plus the custom file / URL inputs of `RepositoryDialog.qml`. */
import { Box } from '@mui/material'

import { ImButton } from '@/components/ImButton'
import { ImRadioButton } from '@/components/ImRadioButton'
import { ImTextField } from '@/components/ImTextField'
import { useTr } from '@/i18n'
import { spacing } from '@/theme/tokens'

import { displayUrl, type RepoSource, type useRepository } from './useRepository'

type Repository = ReturnType<typeof useRepository>

function CustomFile({ repository }: { repository: Repository }) {
  const tr = useTr('RepositoryDialog')
  const common = useTr('CommonStrings')
  return (
    <Box sx={{ display: 'flex', gap: `${spacing.medium}px`, alignItems: 'center' }}>
      <ImTextField
        fullWidth
        autoFocus
        value={repository.repo.file ? displayUrl(repository.repo.file) : ''}
        placeholder={tr('Please select a custom repository json file')}
        slotProps={{ htmlInput: { readOnly: true, 'aria-label': tr('Use custom file') } }}
      />
      <ImButton
        onClick={() => void repository.browse()}
        sx={{ minWidth: 80 }}
        accessibleDescription={tr('Select a custom repository JSON file from your computer')}
      >
        {common('Browse')}
      </ImButton>
    </Box>
  )
}

export function RepoSourceFields({ repository }: { repository: Repository }) {
  const tr = useTr('RepositoryDialog')
  const { repo, change } = repository
  const radio = (source: RepoSource, label: string, description: string) => (
    <ImRadioButton
      name="repo-source"
      value={source}
      label={label}
      accessibleDescription={description}
      checked={repo.source === source}
      onSelect={() => change({ source })}
    />
  )
  return (
    <>
      {radio(
        'official',
        'Raspberry Pi (default)',
        tr('Use the official Raspberry Pi operating system repository'),
      )}
      {radio(
        'file',
        tr('Use custom file'),
        tr('Load operating system list from a JSON file on your computer'),
      )}
      {radio(
        'url',
        tr('Use custom URL'),
        tr('Download operating system list from a custom web address'),
      )}
      {repo.source === 'file' ? <CustomFile repository={repository} /> : null}
      {repo.source === 'url' ? (
        <ImTextField
          fullWidth
          autoFocus
          value={repo.uri}
          placeholder="https://example.com/repo.json"
          onChange={(event) => change({ uri: event.target.value })}
          slotProps={{ htmlInput: { inputMode: 'url', 'aria-label': tr('Use custom URL') } }}
        />
      ) : null}
    </>
  )
}
