import { theme } from '@/theme/theme'
import { colors, fontSize, sizes, spacing } from '@/theme/tokens'

// Values copied verbatim from upstream src/Style.qml (v2.0.7).
describe('Style.qml tokens', () => {
  it('keeps the upstream colours', () => {
    expect(colors.raspberryRed).toBe('#ab1e3a')
    expect(colors.buttonFocusedBackground).toBe('#d1dcfb')
    expect(colors.buttonHoveredBackground).toBe('#f2f2f2')
    expect(colors.button2HoveredBackground).toBe('#eac7ce')
    expect(colors.titleBackground).toBe('#f5f5f5')
    expect(colors.popupBorder).toBe('#dcdcdc')
    expect(colors.listViewHighlight).toBe('#BACCE7')
    expect(colors.textMetadata).toBe('#646464')
    expect(colors.formControlActive).toBe('#1955AE')
    expect(colors.focusOutline).toBe('#0078d4')
    expect(colors.progressBarVerifyForeground).toBe('#6cc04a')
  })

  it('keeps the upstream type scale and spacing', () => {
    expect(fontSize).toMatchObject({ xs: 12, sm: 14, md: 16, xl: 24 })
    expect(spacing).toMatchObject({ xSmall: 5, small: 10, medium: 15, large: 20 })
    expect(sizes).toMatchObject({
      buttonHeight: 40,
      buttonMinWidth: 120,
      sidebarWidth: 200,
      sidebarMinWidth: 150,
      sidebarMaxWidth: 350,
      sidebarSubItemHeight: 28,
    })
  })
})

describe('MUI theme', () => {
  it('uses Roboto at a 14px base on a light palette', () => {
    expect(theme.palette.mode).toBe('light')
    expect(theme.palette.primary.main).toBe('#ab1e3a')
    expect(theme.typography.fontFamily).toMatch(/^Roboto/)
    expect(theme.typography.fontSize).toBe(14)
  })

  it('disables ripple globally', () => {
    expect(theme.components?.MuiButtonBase?.defaultProps?.disableRipple).toBe(true)
  })
})
