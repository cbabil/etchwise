/**
 * Type-to-filter matching from upstream `ImComboBox.qml`: an option
 * matches when it starts with the search string or the search string
 * starts a word (after whitespace, comma, parenthesis or slash).
 */

export interface FilteredOption {
  text: string
  originalIndex: number
}

/** True when `search` (lower case) starts a word inside `text`. */
export function wordBoundaryMatch(text: string, search: string): boolean {
  const lower = text.toLowerCase()
  let pos = 0
  while (pos <= lower.length - search.length) {
    const idx = lower.indexOf(search, pos)
    if (idx === -1) return false
    if (idx === 0 || /[\s,()/]/.test(lower.charAt(idx - 1))) return true
    pos = idx + 1
  }
  return false
}

/** Options matching `search`, keeping their original indices. */
export function filterOptions(options: readonly string[], search: string): FilteredOption[] {
  const needle = search.toLowerCase()
  return options
    .map((text, originalIndex) => ({ text, originalIndex }))
    .filter(
      ({ text }) =>
        needle.length === 0 ||
        text.toLowerCase().startsWith(needle) ||
        wordBoundaryMatch(text, needle),
    )
}

/** Popup height: 40px rows, between 150 and 300px, never past the window. */
export function popupHeight(count: number, available: number, itemHeight = 40): number {
  const ideal = Math.min(300, Math.max(150, count * itemHeight))
  return Math.max(itemHeight * 2, Math.min(ideal, available))
}
