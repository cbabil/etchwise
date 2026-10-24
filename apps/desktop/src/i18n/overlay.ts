/**
 * Etchwise's own translations, layered over the upstream catalogues in
 * `locales/` (which `scripts/convert-translations.py` regenerates from
 * Raspberry Pi Imager and would overwrite), kept in `overlay.json` in the
 * catalogues' shape:
 * {language: {context: {source: translation}}}. Wording follows each
 * catalogue's existing style (formal or informal address, verb form); every
 * source here needs an entry for every non-English language (tested).
 */

import overlayData from './overlay.json'

export type Catalogue = Record<string, Record<string, string>>

/** The overlay data: `overlay.json`, keyed by language. */
export const OVERLAY: Readonly<Record<string, Catalogue>> = overlayData

/** `catalogue` with the overlay's entries for `language` added per context. */
export function withOverlay(language: string, catalogue: Catalogue): Catalogue {
  const extra = OVERLAY[language]
  if (!extra) return catalogue
  const merged: Catalogue = { ...catalogue }
  for (const [context, entries] of Object.entries(extra)) {
    merged[context] = { ...catalogue[context], ...entries }
  }
  return merged
}
