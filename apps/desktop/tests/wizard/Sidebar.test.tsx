import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react'

import * as imager from '@/api/imager'
import { Sidebar } from '@/wizard/Sidebar'
import { SidebarResizeHandle } from '@/wizard/SidebarResizeHandle'
import { Step } from '@/wizard/stepIds'
import { type WizardApi, WizardProvider } from '@/wizard/WizardContext'

import { Capture } from './capture'

vi.mock('@/api/imager', async (original) => {
  const actual = await original<typeof import('@/api/imager')>()
  return { ...actual, call: vi.fn(actual.call) }
})

let wizard: WizardApi
const keep = (w: WizardApi) => {
  wizard = w
}

async function renderSidebar(onOpenAppOptions = vi.fn()) {
  render(
    <WizardProvider>
      <Capture onWizard={keep} />
      <Sidebar onOpenAppOptions={onOpenAppOptions} />
      <SidebarResizeHandle />
    </WizardProvider>,
  )
  await waitFor(() => expect(wizard.ready).toBe(true))
  return onOpenAppOptions
}

const row = (name: string) => screen.getByText(name).closest('[role="button"]') as HTMLElement

describe('Sidebar', () => {
  it('lists the step groups with the current one active', async () => {
    const onOpen = await renderSidebar()
    const nav = screen.getByRole('navigation', { name: 'Setup steps' })
    expect(within(nav).getByRole('heading', { name: 'Setup steps' })).toBeInTheDocument()
    for (const name of ['Device', 'OS', 'Storage', 'Customisation', 'Writing', 'Done'])
      expect(row(name)).toBeInTheDocument()
    expect(row('Device')).toHaveAttribute('aria-current', 'step')
    expect(getComputedStyle(row('Device')).backgroundColor).toBe('rgb(171, 30, 58)')
    expect(getComputedStyle(screen.getByText('OS')).color).toBe('rgb(128, 128, 128)')
    expect(getComputedStyle(nav).width).toBe('200px')
    fireEvent.click(screen.getByRole('button', { name: /^App Options/ }))
    expect(onOpen).toHaveBeenCalledOnce()
  })

  it('navigates back through visited rows only, and shows sub-steps after OS', async () => {
    await renderSidebar()
    fireEvent.click(row('Storage'))
    expect(wizard.currentStep).toBe(Step.Device)
    act(() => wizard.next())
    act(() => wizard.next())
    expect(screen.getByTestId('sidebar-sub-Hostname')).toBeInTheDocument()
    expect(getComputedStyle(screen.getByText('Device')).color).toBe('rgb(171, 30, 58)')
    fireEvent.click(row('Device'))
    expect(wizard.currentStep).toBe(Step.Device)
    expect(screen.queryByTestId('sidebar-sub-Hostname')).toBeNull()
  })

  it('jumps between customisation sub-steps, bold when configured, underlined when current', async () => {
    await renderSidebar()
    act(() => wizard.jumpTo(Step.Locale))
    act(() => wizard.update({ hostnameConfigured: true }))
    const hostname = screen.getByTestId('sidebar-sub-Hostname')
    expect(getComputedStyle(within(hostname).getByText('Hostname')).fontWeight).toBe('700')
    expect(screen.getByTestId('sidebar-sub-Localisation')).toHaveAttribute('aria-current', 'step')
    fireEvent.click(screen.getByTestId('sidebar-sub-User'))
    expect(wizard.currentStep).toBe(Step.Locale)
    fireEvent.click(hostname)
    expect(wizard.currentStep).toBe(Step.Hostname)
    act(() => wizard.update({ isWriting: true }))
    fireEvent.click(screen.getByTestId('sidebar-sub-Localisation'))
    expect(wizard.currentStep).toBe(Step.Hostname)
  })

  it('resizes by dragging within limits, persists on release and resets on double-click', async () => {
    await renderSidebar()
    const handle = screen.getByRole('separator', { name: 'Sidebar resize handle' })
    const nav = screen.getByRole('navigation')
    fireEvent.pointerMove(handle, { clientX: 400 })
    fireEvent.pointerUp(handle)
    expect(nav).toHaveStyle({ width: '200px' })
    fireEvent.pointerEnter(handle)
    fireEvent.pointerDown(handle, { clientX: 100, pointerId: 1 })
    fireEvent.pointerMove(handle, { clientX: 160 })
    expect(nav).toHaveStyle({ width: '260px' })
    fireEvent.pointerMove(handle, { clientX: 900 })
    expect(nav).toHaveStyle({ width: '350px' })
    fireEvent.pointerUp(handle)
    fireEvent.pointerLeave(handle)
    const saves = vi.mocked(imager.call).mock.calls.filter(([m]) => m === 'setSetting')
    expect(saves.at(-1)).toEqual(['setSetting', 'sidebarWidth', '350'])
    fireEvent.doubleClick(handle)
    expect(nav).toHaveStyle({ width: '200px' })
  })
})
