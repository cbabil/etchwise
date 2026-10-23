/**
 * The 8px drag handle right of the sidebar: drag to resize within
 * 150-350px, release to persist ("sidebarWidth" setting), double-click to
 * restore 200px. A 1px separator spans 75% of the height.
 */
import { Box } from '@mui/material'

import { useTr } from '@/i18n'
import { colors, sizes } from '@/theme/tokens'

import { useSidebarDrag } from './useSidebarDrag'

/** Style.qml `sidebarDragHandleHoverBackground`: #f5f5f5 at 30%. */
const hoverBackground = 'rgba(245, 245, 245, 0.3)'

export function SidebarResizeHandle() {
  const tr = useTr('WizardContainer')
  const { handlers, active } = useSidebarDrag()
  return (
    <Box
      role="separator"
      aria-orientation="vertical"
      aria-label={tr('Sidebar resize handle')}
      {...handlers}
      sx={{
        width: sizes.sidebarDragHandleWidth,
        flexShrink: 0,
        height: '100%',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        cursor: 'col-resize',
        backgroundColor: active ? hoverBackground : 'transparent',
        touchAction: 'none',
      }}
    >
      <Box
        sx={{
          width: '1px',
          height: '75%',
          backgroundColor: active ? colors.listViewHoverRowBackground : colors.titleSeparator,
        }}
      />
    </Box>
  )
}
