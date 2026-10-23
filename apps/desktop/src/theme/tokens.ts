/**
 * Design tokens copied verbatim from upstream Raspberry Pi Imager
 * `src/Style.qml` (v2.0.7). Sizes are CSS px at a text scale of 1.0,
 * which matches Qt point sizes on macOS (font DPI correction 1.0). Qt named
 * colours (white, black, red, grey) are written as their exact hex values.
 */

const raspberryRed = '#ab1e3a'
const mainBackground = '#ffffff'
const titleBackground = '#f5f5f5'

export const colors = {
  mainBackground,
  raspberryRed,
  raspberryRedHighlight: '#d64561',
  buttonBackground: mainBackground,
  buttonForeground: raspberryRed,
  buttonFocusedBackground: '#d1dcfb',
  buttonHoveredBackground: '#f2f2f2',
  button2Background: raspberryRed,
  button2Foreground: mainBackground,
  button2FocusedBackground: '#8f122c',
  button2HoveredBackground: '#eac7ce',
  button2HoveredForeground: raspberryRed,
  titleBackground,
  titleSeparator: '#afafaf',
  popupBorder: '#dcdcdc',
  listViewRowBackground: '#ffffff',
  listViewHoverRowBackground: titleBackground,
  listViewHighlight: '#BACCE7',
  textDescription: '#1a1a1a',
  textMetadata: '#646464',
  sidebarActiveBackground: raspberryRed,
  sidebarTextOnActive: '#FFFFFF',
  sidebarTextOnInactive: raspberryRed,
  sidebarTextDisabled: '#E0E0E0',
  sidebarControlBorder: '#767676',
  sidebarBackground: mainBackground,
  sidebarBorder: raspberryRed,
  progressBarText: '#ffffff',
  progressBarVerifyForeground: '#6cc04a',
  progressBarWritingForeground: raspberryRed,
  progressBarTrack: titleBackground,
  lanbarBackground: '#ffffe3',
  formLabel: '#000000',
  formLabelError: '#ff0000',
  formLabelDisabled: '#808080',
  formControlActive: '#1955AE',
  focusOutline: '#0078d4',
  /** Hard-coded outside Style.qml: write warning and OTP warning text. */
  writeWarning: '#FFA500',
  otpWarning: '#C62828',
  dialogBackdrop: 'rgba(0, 0, 0, 0.3)',
} as const

export const fontSize = { xs: 12, sm: 14, md: 16, xl: 24 } as const

export const spacing = {
  xxSmall: 2,
  xSmall: 5,
  tiny: 8,
  small: 10,
  smallPlus: 12,
  medium: 15,
  large: 20,
  extraLarge: 30,
} as const

export const sizes = {
  buttonHeight: 40,
  /** Qt Material Button `topInset` / `bottomInset`: the background is drawn
   * this far inside the 40px button, so it shows 28px tall. */
  buttonInset: 6,
  buttonMinWidth: 120,
  buttonSkipWidth: 150,
  buttonRadius: 4,
  sectionMaxWidth: 500,
  sectionPadding: 16,
  sectionBorderRadius: 8,
  listItemBorderRadius: 5,
  listItemPadding: 15,
  listIcon: 40,
  cardPadding: 20,
  scrollBarWidth: 10,
  sidebarWidth: 200,
  sidebarMinWidth: 150,
  sidebarMaxWidth: 350,
  sidebarDragHandleWidth: 8,
  sidebarItemHeight: 40,
  sidebarSubItemHeight: 28,
  sidebarItemRadius: 4,
  formColumnSpacing: 20,
  formRowSpacing: 15,
  stepContentMargins: 24,
  stepContentSpacing: 16,
  focusOutlineWidth: 2,
  focusOutlineRadius: 4,
  dialogWidth: 400,
  dialogPadding: 20,
} as const

export const fontFamily = 'Roboto, system-ui, -apple-system, sans-serif'
