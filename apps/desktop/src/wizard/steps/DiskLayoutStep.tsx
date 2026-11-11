/**
 * Etchwise, not upstream: Kura's disk layout, offered only for Kura images
 * and saved as `kuraStorage` (Kura's `storage` key in kura-setup.json).
 * Simple keeps one root that fills the disk; Server and Custom size the
 * root and /tmp and give folders their own volumes. Kura checks the layout
 * against the real disk on first boot and keeps one root if it doesn't fit.
 */
import { Box } from '@mui/material'
import { useState } from 'react'

import { ImRadioButton } from '@/components/ImRadioButton'
import { formatSize } from '@/format'
import { useTr } from '@/i18n'
import { colors, fontSize, spacing } from '@/theme/tokens'

import { StepBase } from '../StepBase'
import { useWizard, type WizardApi } from '../WizardContext'
import { DescriptionText, SectionContainer, StepContent } from './custom/layout'
import { editSettings, persist, unpersist } from './custom/settings'
import { useCustomStepNav } from './custom/useCustomStepNav'
import {
  DISK_CONTEXT as CONTEXT,
  fromSettings,
  type Layout,
  layoutError,
  neededBytes,
  type Preset,
  toStorage,
  withPreset,
} from './disk/layout'
import { VolumeRows } from './disk/VolumeRows'
import { selectedDrive } from './storage/selectedDrive'

/** Stores the layout at runtime and persistently. */
export function saveLayout(wizard: WizardApi, layout: Layout): void {
  const storage = toStorage(layout)
  editSettings(
    wizard,
    { kuraDiskLayout: layout.preset, ...(storage ? { kuraStorage: storage } : {}) },
    storage ? [] : ['kuraStorage'],
  )
  persist('kuraDiskLayout', layout.preset)
  if (storage) persist('kuraStorage', storage)
  else unpersist('kuraStorage')
}

const PRESETS: readonly { preset: Preset; label: string; help: string }[] = [
  { preset: 'simple', label: 'Simple', help: 'One root partition that fills the disk.' },
  {
    preset: 'server',
    label: 'Server (recommended)',
    help: 'Logs, temporary files and homes on their own volumes, so they cannot fill the system; Docker gets the rest of the disk.',
  },
  { preset: 'custom', label: 'Custom', help: 'Choose the folders and their sizes.' },
]

function Fit({ layout }: { layout: Layout }) {
  const tr = useTr(CONTEXT)
  const needed = neededBytes(layout)
  if (needed === 0) return null
  const error = layoutError(layout)
  const drive = selectedDrive()?.size ?? 0
  const small = drive > 0 && drive < needed
  const sx = { fontSize: fontSize.sm, color: colors.formLabelError }
  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', gap: `${spacing.xxSmall}px` }}>
      {error ? <Box sx={sx}>{tr(error)}</Box> : null}
      <DescriptionText>{tr('Needs a drive of at least %1.', formatSize(needed))}</DescriptionText>
      {small ? (
        <Box sx={sx}>
          {tr(
            "The selected drive (%1) is smaller than this layout needs. Written to it, Kura keeps one root that fills the disk; a PC installer checks the PC's own drive instead.",
            formatSize(drive),
          )}
        </Box>
      ) : null}
    </Box>
  )
}

export function DiskLayoutStep() {
  const tr = useTr(CONTEXT)
  const wizard = useWizard()
  const nav = useCustomStepNav(CONTEXT, 'Save the disk layout and continue')
  const [layout, setLayout] = useState(() => fromSettings(wizard.customizationSettings))
  return (
    <StepBase
      title={tr('Customisation: Disk layout')}
      {...nav}
      nextEnabled={layoutError(layout) === ''}
      onNext={() => {
        saveLayout(wizard, layout)
        wizard.next()
      }}
    >
      <StepContent>
        <SectionContainer>
          <Box role="radiogroup" sx={{ display: 'flex', flexDirection: 'column' }}>
            {PRESETS.map(({ preset, label, help }) => (
              <ImRadioButton
                key={preset}
                label={tr(label)}
                name="disk-layout"
                checked={layout.preset === preset}
                accessibleDescription={tr(help)}
                onSelect={() => setLayout(withPreset(layout, preset))}
              />
            ))}
          </Box>
          <DescriptionText>
            {tr(PRESETS.find((p) => p.preset === layout.preset)?.help ?? '')}
          </DescriptionText>
          {layout.preset === 'simple' ? null : <VolumeRows layout={layout} onChange={setLayout} />}
          <Fit layout={layout} />
        </SectionContainer>
      </StepContent>
    </StepBase>
  )
}
