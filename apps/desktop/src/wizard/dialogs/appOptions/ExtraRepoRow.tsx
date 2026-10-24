/**
 * App Options row for the extra repository: Choose / Remove, with the
 * chosen file's name below, or why it did not load.
 */
import { Box } from '@mui/material'

import { colors, fontSize } from '@/theme/tokens'

import { ImOptionButton } from '../common/ImOptionButton'
import { displayUrl } from '../repository/useRepository'
import type { ExtraRepo } from './useExtraRepo'

const fileName = (url: string) => displayUrl(url).split('/').pop() ?? url

export function ExtraRepoRow({ repo, disabled }: { repo: ExtraRepo; disabled: boolean }) {
  const name = repo.url ? fileName(repo.url) : ''
  return (
    <Box>
      <ImOptionButton
        label="Extra Repository"
        buttonText={repo.url ? 'Remove' : 'Choose'}
        accessibleDescription="Add the devices and operating systems of another repository file to the official list"
        disabled={disabled}
        onClick={repo.url ? repo.remove : () => void repo.choose()}
      />
      {name ? (
        <Box
          sx={{
            fontSize: fontSize.xs,
            color: repo.error ? colors.formLabelError : colors.textMetadata,
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
        >
          {repo.error ? `Could not load ${name}: ${repo.error}` : name}
        </Box>
      ) : null}
    </Box>
  )
}
