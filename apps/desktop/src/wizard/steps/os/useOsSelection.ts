/**
 * Upstream `handleOSSelection` / `selectOSitem`: groups open a sublist,
 * "Back" closes it, Erase / an image become the source (keyboard and
 * double-click selections then advance), and "Use custom" opens the file
 * picker. Upstream assigns `nextButtonEnabled` imperatively here, which
 * `nextOverride` mirrors.
 */
import { useRef, useState } from 'react'

import { useTr } from '@/i18n'

import type { WizardApi } from '../../WizardContext'
import { applyCustomImage, chooseImageFile, imageFilterPattern } from './customImage'
import { useEntryText } from './OsRow'
import { selectErase, selectOs } from './selectImage'
import { isBackRow, isSublist, type OsEntry, URL_CUSTOM, URL_FORMAT } from './types'
import type { OsListState } from './useOsList'

export interface OsSelection {
  nextOverride: boolean | null
  customSize: number
  handle: (entry: OsEntry, fromKeyboard: boolean) => Promise<void>
}

function useCustomPicker(wizard: WizardApi, list: OsListState) {
  const tr = useTr('OSSelectionStep')
  const common = useTr('CommonStrings')
  const [customSize, setCustomSize] = useState(0)
  const pick = async (setOverride: (v: boolean) => void) => {
    const path = await chooseImageFile({
      title: tr('Select image'),
      images: common('Image files (%1)', imageFilterPattern),
      all: common('All files (*)'),
    })
    if (!path) return
    setCustomSize(await applyCustomImage(wizard, path))
    setOverride(true)
    const index = list.levels[0]?.entries.findIndex((e) => e.url === URL_CUSTOM) ?? -1
    if (index !== -1 && list.levels.length === 1) list.setCurrent(index)
  }
  return { customSize, setCustomSize, pick }
}

export function useOsSelection(wizard: WizardApi, list: OsListState): OsSelection {
  const [nextOverride, setNextOverride] = useState<boolean | null>(null)
  const category = useRef('')
  const text = useEntryText()
  const custom = useCustomPicker(wizard, list)
  const handle = async (entry: OsEntry, fromKeyboard: boolean) => {
    if (isSublist(entry)) {
      category.current = entry.name
      await list.openGroup(entry)
      return
    }
    if (isBackRow(entry)) {
      list.goBack()
      category.current = ''
      return
    }
    if (entry.url === URL_CUSTOM) {
      setNextOverride(false)
      await custom.pick(setNextOverride)
      return
    }
    if (entry.url === URL_FORMAT) {
      await selectErase(wizard, entry, text(entry).name, category.current)
    } else {
      await selectOs(wizard, entry, category.current)
      custom.setCustomSize(0)
    }
    setNextOverride(true)
    if (fromKeyboard) wizard.next()
  }
  return { nextOverride, customSize: custom.customSize, handle }
}
