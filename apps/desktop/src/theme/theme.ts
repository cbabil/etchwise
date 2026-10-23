/**
 * MUI theme reproducing upstream's Qt Material "Dense" look from the
 * Style.qml tokens. Ripple and click effects are disabled (house rule).
 */
import { createTheme } from '@mui/material/styles'

import { colors, fontFamily, fontSize, sizes } from './tokens'

export const theme = createTheme({
  palette: {
    mode: 'light',
    primary: { main: colors.raspberryRed, contrastText: colors.button2Foreground },
    secondary: { main: colors.formControlActive },
    error: { main: colors.formLabelError },
    background: { default: colors.mainBackground, paper: colors.mainBackground },
    text: { primary: colors.formLabel, secondary: colors.textMetadata },
    divider: colors.popupBorder,
  },
  typography: {
    fontFamily,
    fontSize: fontSize.sm,
    htmlFontSize: 16,
    button: { fontSize: fontSize.sm, textTransform: 'uppercase', fontWeight: 500 },
  },
  shape: { borderRadius: sizes.buttonRadius },
  components: {
    MuiButtonBase: { defaultProps: { disableRipple: true, disableTouchRipple: true } },
    MuiButton: {
      defaultProps: { disableElevation: true },
      styleOverrides: {
        root: {
          height: sizes.buttonHeight - 2 * sizes.buttonInset,
          marginBlock: sizes.buttonInset,
          minWidth: sizes.buttonMinWidth,
          transition: 'none',
          boxShadow: 'none',
        },
      },
    },
    MuiIconButton: {
      styleOverrides: {
        root: { backgroundColor: 'transparent', '&:hover': { backgroundColor: 'transparent' } },
      },
    },
  },
})
