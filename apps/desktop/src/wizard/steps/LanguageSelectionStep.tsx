/**
 * Upstream `LanguageSelectionStep.qml`: a centred "Language:" drop-down
 * of the bundled translations (native names sorted case-insensitively,
 * "English" shown as "English (British)"). Choosing one switches the UI
 * at once; Next saves it as `savedLanguage` and moves on to the Device
 * step, or the OS step while offline.
 */
import { Box } from '@mui/material'
import { useEffect, useMemo, useState } from 'react'

import { call } from '@/api/imager'
import { ImComboBox } from '@/components/ImComboBox'
import { useLanguage, useTr } from '@/i18n'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

import { StepBase } from '../StepBase'
import { Step } from '../stepIds'
import { useWizard } from '../WizardContext'

const BRITISH = 'English (British)'

const localeFiles = import.meta.glob('../../i18n/locales/*.json')

/** Qt's `nativeLanguageName` where `Intl` differs. */
const QT_NAMES: Readonly<Record<string, string>> = { zh: '简体中文', 'zh-TW': '繁體中文' }

function nativeName(code: string): string {
  if (code === 'en') return 'English'
  if (QT_NAMES[code]) return QT_NAMES[code]
  try {
    return new Intl.DisplayNames([code], { type: 'language' }).of(code) ?? code
  } catch {
    return code
  }
}

export interface Translation {
  name: string
  code: string
}

/** Upstream `getTranslations()`: name to code, sorted case-insensitively. */
export function translations(): readonly Translation[] {
  const codes = Object.keys(localeFiles).map((p) => p.replace(/^.*\/([^/]+)\.json$/, '$1'))
  const fold = (s: string) => s.toLowerCase()
  return codes
    .map((code) => ({ name: nativeName(code), code }))
    .sort((a, b) => (fold(a.name) < fold(b.name) ? -1 : fold(a.name) > fold(b.name) ? 1 : 0))
}

const display = (name: string) => (name === 'English' ? BRITISH : name)

function useLanguageChoice() {
  const { language, setLanguage } = useLanguage()
  const langs = useMemo(() => translations(), [])
  const options = useMemo(() => langs.map((l) => display(l.name)), [langs])
  const [index, setIndex] = useState(-1)
  useEffect(() => {
    let live = true
    void call<string | null>('getCurrentLanguage').then((current) => {
      if (!live) return
      const byName = current ? options.indexOf(display(current)) : -1
      const byCode = langs.findIndex((l) => l.code === language)
      const found = byName !== -1 ? byName : byCode !== -1 ? byCode : options.indexOf(BRITISH)
      setIndex(found !== -1 ? found : 0)
    })
    return () => {
      live = false
    }
    // Only the initial pre-selection depends on the current language.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])
  const apply = (i: number) => {
    const chosen = langs[i]
    if (!chosen) return null
    void call('changeLanguage', chosen.name)
    setLanguage(chosen.code)
    return chosen
  }
  return { options, index, setIndex, apply }
}

export function LanguageSelectionStep() {
  const tr = useTr('LanguageSelectionStep')
  const wizard = useWizard()
  const { options, index, setIndex, apply } = useLanguageChoice()
  const onNext = () => {
    const chosen = apply(index)
    if (chosen) void call('setSetting', 'savedLanguage', chosen.name)
    wizard.jumpTo(wizard.hasNetworkConnectivity ? Step.Device : Step.OS)
  }
  return (
    <StepBase
      title={tr('Welcome')}
      subtitle={tr('Choose your language for Raspberry Pi Imager')}
      showBack={false}
      onNext={onNext}
    >
      <Box sx={{ flex: 1, display: 'flex', alignItems: 'center', p: `${sizes.sectionPadding}px` }}>
        <Box
          sx={{
            width: '100%',
            maxWidth: sizes.sectionMaxWidth,
            mx: 'auto',
            display: 'flex',
            alignItems: 'center',
            gap: `${spacing.medium}px`,
          }}
        >
          <Box component="label" sx={{ fontSize: fontSize.sm, color: colors.formLabel }}>
            {tr('Language:')}
          </Box>
          <Box sx={{ flex: 1, minWidth: 0 }}>
            <ImComboBox
              options={options}
              value={index}
              ariaLabel={`${tr('Language:')} ${options[index] ?? ''}. ${tr('Select the language for the Raspberry Pi Imager interface')}`}
              onChange={(i) => {
                setIndex(i)
                apply(i)
              }}
            />
          </Box>
        </Box>
      </Box>
    </StepBase>
  )
}
