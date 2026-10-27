import { act, fireEvent, render, renderHook, screen, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'

import * as imager from '@/api/imager'
import { mockCall, mockEmit } from '@/api/mock'
import { Step } from '@/wizard/stepIds'
import { useWizard, WizardProvider, type WizardProviderProps } from '@/wizard/WizardContext'

vi.mock('@/api/imager', async (original) => {
  const actual = await original<typeof import('@/api/imager')>()
  return { ...actual, call: vi.fn(actual.call) }
})

const calls = (method: string) => vi.mocked(imager.call).mock.calls.filter(([m]) => m === method)

async function setup(props: Partial<WizardProviderProps> = {}) {
  const wrapper = ({ children }: { children: ReactNode }) => (
    <WizardProvider {...props}>{children}</WizardProvider>
  )
  const hook = renderHook(() => useWizard(), { wrapper })
  await waitFor(() => expect(hook.result.current.ready).toBe(true))
  return hook
}

describe('WizardProvider start-up', () => {
  it('loads saved customisation, sidebar width and starts at Device', async () => {
    mockCall('setSavedCustomisationSettings', [{ hostname: 'pi' }])
    mockCall('setSetting', ['sidebarWidth', '999'])
    mockCall('setSetting', ['secureboot_rsa_key', '/key.pem'])
    const { result } = await setup()
    expect(result.current.currentStep).toBe(Step.Device)
    expect(result.current.customizationSettings).toEqual({ hostname: 'pi' })
    expect(result.current.sidebarWidth).toBe(350)
    expect(result.current.secureBootKeyConfigured).toBe(true)
    expect(result.current.totalSteps).toBe(13)
    expect(result.current.lastCustomizationStep).toBe(Step.RemoteAccess)
  })

  it('starts at Language when requested and at OS while offline', async () => {
    expect((await setup({ showLanguageSelection: true })).result.current.currentStep).toBe(
      Step.Language,
    )
    vi.mocked(imager.call).mockImplementation(async (method: string, ...args: unknown[]) =>
      method === 'isOsListUnavailable' ? true : mockCall(method, args),
    )
    const offline = await setup({ showLanguageSelection: false })
    vi.mocked(imager.call).mockImplementation(async (method: string, ...args: unknown[]) =>
      mockCall(method, args),
    )
    expect(offline.result.current).toMatchObject({
      hasNetworkConnectivity: false,
      currentStep: Step.OS,
    })
    act(() => offline.result.current.back())
    act(() => offline.result.current.jumpTo(Step.Device))
    expect(offline.result.current.currentStep).toBe(Step.OS)
  })

  it('throws outside the provider', () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
    expect(() => renderHook(() => useWizard())).toThrow(/WizardProvider/)
  })
})

describe('WizardProvider navigation', () => {
  it('next / back / jumpTo and customisation apply on entering Writing', async () => {
    const { result } = await setup()
    act(() => result.current.next())
    expect(result.current.currentStep).toBe(Step.OS)
    act(() => result.current.back())
    expect(result.current.currentStep).toBe(Step.Device)
    act(() => result.current.jumpTo(Step.RemoteAccess))
    act(() => result.current.update({ wifiConfigured: true }))
    act(() => result.current.next())
    expect(result.current.currentStep).toBe(Step.Writing)
    expect(calls('applyCustomisationFromSettings').at(-1)?.[1]).toMatchObject({
      wifiConfigured: true,
    })
    expect(result.current.isStepPermissible(Step.RemoteAccess)).toBe(true)
    expect(result.current.isStepPermissible(Step.Done)).toBe(false)
  })

  it('skipCustomisation jumps to Writing with flags cleared', async () => {
    const { result } = await setup()
    act(() => result.current.jumpTo(Step.SecureBoot))
    act(() => result.current.update({ hostnameConfigured: true, secureBootEnabled: true }))
    act(() => result.current.skipCustomisation({ secureBootEnabled: false }))
    expect(result.current).toMatchObject({
      currentStep: Step.Writing,
      hostnameConfigured: false,
      secureBootEnabled: false,
    })
  })

  it('selection setters invalidate dependent steps', async () => {
    const { result } = await setup()
    act(() => result.current.setSelectedDeviceName('Pi 5', true))
    act(() => result.current.next())
    act(() => result.current.setSelectedOsName('RPi OS'))
    act(() => result.current.next())
    act(() => result.current.setSelectedStorageName('SD'))
    act(() => result.current.setSelectedDeviceName('Pi 4', true))
    expect(result.current).toMatchObject({ selectedOsName: '', selectedStorageName: '' })
    expect(result.current.isStepPermissible(Step.Storage)).toBe(false)
  })

  it('edits customisation settings immutably', async () => {
    const { result } = await setup()
    act(() => result.current.setCustomizationSettings({ a: 1 }))
    const before = result.current.customizationSettings
    act(() => result.current.updateCustomizationSettings({ b: 2 }))
    act(() => result.current.removeCustomizationSetting('a'))
    expect(before).toEqual({ a: 1 })
    expect(result.current.customizationSettings).toEqual({ b: 2 })
  })

  it('reset and Write Another tell the backend', async () => {
    const { result } = await setup()
    act(() => result.current.jumpTo(Step.Done))
    act(() => result.current.resetToWriteStep())
    expect(result.current).toMatchObject({ currentStep: Step.Storage, writeAnotherMode: true })
    act(() => result.current.next())
    expect(result.current.currentStep).toBe(Step.Writing)
    act(() => result.current.resetWizard())
    expect(result.current.currentStep).toBe(Step.Device)
    expect(calls('setHWIndex').at(-1)).toEqual(['setHWIndex', -1])
    expect(calls('setDst').length).toBeGreaterThanOrEqual(2)
  })

  it('persists the sidebar width only when asked', async () => {
    const { result } = await setup()
    act(() => result.current.setSidebarWidth(100))
    expect(result.current.sidebarWidth).toBe(150)
    act(() => result.current.setSidebarWidth(260, true))
    expect(calls('setSetting').at(-1)).toEqual(['setSetting', 'sidebarWidth', '260'])
  })
})

describe('WizardProvider signals', () => {
  it('moves from OS to Device when the OS list becomes available, and to Writing on cancel', async () => {
    const { result } = await setup()
    act(() => result.current.jumpTo(Step.OS))
    act(() => mockEmit({ name: 'osListUnavailableChanged', data: null }))
    await waitFor(() => expect(result.current.currentStep).toBe(Step.Device))
    act(() => mockEmit({ name: 'cancelled', data: null }))
    expect(result.current.currentStep).toBe(Step.Writing)
    act(() => mockEmit({ name: 'cancelled', data: null }))
    expect(result.current.currentStep).toBe(Step.Writing)
  })

  it('stays put while writing', async () => {
    const { result } = await setup()
    act(() => result.current.jumpTo(Step.OS))
    act(() => result.current.update({ isWriting: true }))
    act(() => mockEmit({ name: 'osListUnavailableChanged', data: null }))
    await waitFor(() => expect(calls('isOsListUnavailable').length).toBeGreaterThan(1))
    expect(result.current.currentStep).toBe(Step.OS)
  })
})

function Probe() {
  const wizard = useWizard()
  return (
    <button type="button" onClick={wizard.next}>
      step {wizard.currentStep}
    </button>
  )
}

describe('WizardProvider unmount', () => {
  it('drops subscriptions and late start-up results', async () => {
    const { unmount } = render(
      <WizardProvider>
        <Probe />
      </WizardProvider>,
    )
    unmount()
    await act(async () => mockEmit({ name: 'cancelled', data: null }))
    render(
      <WizardProvider>
        <Probe />
      </WizardProvider>,
    )
    fireEvent.click(await screen.findByRole('button', { name: 'step 0' }))
    expect(screen.getByRole('button', { name: 'step 1' })).toBeInTheDocument()
  })
})
