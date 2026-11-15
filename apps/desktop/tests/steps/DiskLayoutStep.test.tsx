import { fireEvent, screen } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { DiskLayoutStep } from '@/wizard/steps/DiskLayoutStep'
import { setDst } from '@/wizard/steps/storage/selectedDrive'

import { answer, callsTo, nextButton, renderStep } from './customHarness'

vi.mock('@/api/imager', () => ({
  call: vi.fn(async () => null),
  onImagerEvent: vi.fn(async () => () => undefined),
}))

const radio = (name: RegExp) => screen.getByRole('radio', { name })
const size = (name: string) => screen.getByRole('textbox', { name })
const kura = { kura: true, currentStep: Step.DiskLayout }

describe('DiskLayoutStep', () => {
  afterEach(() => setDst('', 0))

  it('starts simple: one root that fills the disk, nothing sent', async () => {
    answer({})
    const { wizard } = await renderStep(DiskLayoutStep, kura)
    expect(radio(/Simple/)).toBeChecked()
    expect(screen.queryByRole('textbox')).toBeNull()
    fireEvent.click(nextButton())
    expect(wizard().customizationSettings).not.toHaveProperty('kuraStorage')
    expect(wizard().customizationSettings.kuraDiskLayout).toBe('simple')
    expect(wizard().currentStep).toBe(Step.Writing)
    expect(callsTo('removePersistedCustomisationSetting')).toEqual([['kuraStorage']])
  })

  it('server shows its sizes, read-only, and how big a drive it needs', async () => {
    answer({})
    await renderStep(DiskLayoutStep, kura)
    fireEvent.click(radio(/Server/))
    expect(size('Root size in GiB')).toHaveValue('16')
    expect(size('Root size in GiB')).toBeDisabled()
    expect(size('/var/log size in GiB')).toHaveValue('4')
    expect(screen.getByText('Needs a drive of at least 31.3 GB.')).toBeInTheDocument()
  })

  it('saves the server layout as Kura storage on Next', async () => {
    answer({})
    const { wizard } = await renderStep(DiskLayoutStep, kura)
    fireEvent.click(radio(/Server/))
    fireEvent.click(nextButton())
    const storage = wizard().customizationSettings.kuraStorage as { rootGiB: number }
    expect(storage.rootGiB).toBe(16)
    expect(callsTo('setPersistedCustomisationSetting')).toContainEqual(['kuraDiskLayout', 'server'])
    expect(callsTo('setPersistedCustomisationSetting')).toContainEqual(['kuraStorage', storage])
  })

  it('custom edits sizes (digits only), adds and removes folders, and blocks Next until valid', async () => {
    answer({})
    const { wizard } = await renderStep(DiskLayoutStep, kura)
    fireEvent.click(radio(/Custom/))
    fireEvent.change(size('Root size in GiB'), { target: { value: '2x' } })
    expect(size('Root size in GiB')).toHaveValue('16')
    fireEvent.change(size('Root size in GiB'), { target: { value: '3' } })
    expect(screen.getByText('The root needs at least 4 GiB.')).toBeInTheDocument()
    expect(nextButton()).toBeDisabled()
    fireEvent.change(size('Root size in GiB'), { target: { value: '20' } })
    expect(nextButton()).toBeEnabled()
    fireEvent.click(screen.getByRole('button', { name: 'Add a folder' }))
    fireEvent.click(screen.getByRole('button', { name: 'Remove /home' }))
    fireEvent.change(size('/var size in GiB'), { target: { value: '50' } })
    fireEvent.click(nextButton())
    expect(wizard().customizationSettings.kuraStorage).toEqual({
      rootGiB: 20,
      tmpGiB: 1,
      volumes: [
        { mount: '/var/log', sizeGiB: 4 },
        { mount: '/var/tmp', sizeGiB: 2 },
        { mount: '/var/lib/docker' },
        { mount: '/var', sizeGiB: 50 },
      ],
    })
  })

  const folders = () => screen.getAllByRole('combobox', { name: 'Folder' })
  const typeFolder = (text: string) => {
    fireEvent.click(screen.getByRole('button', { name: 'Add a folder' }))
    fireEvent.change(folders().at(-1)!, { target: { value: text } })
  }

  it('custom takes any folder typed in', async () => {
    answer({})
    const { wizard } = await renderStep(DiskLayoutStep, kura)
    fireEvent.click(radio(/Custom/))
    typeFolder('/srv/media')
    expect(folders().at(-1)).toHaveValue('/srv/media')
    fireEvent.change(size('/srv/media size in GiB'), { target: { value: '20' } })
    expect(nextButton()).toBeEnabled()
    fireEvent.click(nextButton())
    const storage = wizard().customizationSettings.kuraStorage as { volumes: unknown[] }
    expect(storage.volumes).toContainEqual({ mount: '/srv/media', sizeGiB: 20 })
  })

  it('an invalid folder says why and blocks Next', async () => {
    answer({})
    await renderStep(DiskLayoutStep, kura)
    fireEvent.click(radio(/Custom/))
    typeFolder('/etc/ssh')
    expect(screen.getAllByText('/etc stays on the system disk.').length).toBeGreaterThan(0)
    expect(nextButton()).toBeDisabled()
    fireEvent.change(folders().at(-1)!, { target: { value: 'srv' } })
    expect(screen.getAllByText('Use a folder such as /srv/media.').length).toBeGreaterThan(0)
    expect(nextButton()).toBeDisabled()
  })

  it('saves swap with the layout; empty means none', async () => {
    answer({})
    const { wizard } = await renderStep(DiskLayoutStep, kura)
    fireEvent.click(radio(/Custom/))
    expect(size('Swap size in GiB')).toHaveValue('')
    fireEvent.change(size('Swap size in GiB'), { target: { value: '8' } })
    fireEvent.click(nextButton())
    const storage = wizard().customizationSettings.kuraStorage as { swapGiB: number }
    expect(storage.swapGiB).toBe(8)
    expect(callsTo('setPersistedCustomisationSetting')).toContainEqual(['kuraStorage', storage])
  })

  it('hides the /tmp RAM size while /tmp has its own volume', async () => {
    answer({})
    const { wizard } = await renderStep(DiskLayoutStep, kura)
    fireEvent.click(radio(/Custom/))
    expect(screen.getByText('/tmp (in RAM)')).toBeInTheDocument()
    typeFolder('/tmp')
    expect(screen.queryByText('/tmp (in RAM)')).toBeNull()
    expect(screen.getByText('/tmp is on its own volume.')).toBeInTheDocument()
    expect(screen.getAllByRole('textbox', { name: '/tmp size in GiB' })).toHaveLength(1)
    fireEvent.click(nextButton())
    expect(wizard().customizationSettings.kuraStorage).not.toHaveProperty('tmpGiB')
  })

  it('lets another folder take the rest of the disk', async () => {
    answer({})
    const { wizard } = await renderStep(DiskLayoutStep, kura)
    fireEvent.click(radio(/Custom/))
    fireEvent.click(screen.getByRole('checkbox', { name: '/home takes the rest' }))
    expect(screen.getByText('Only one folder can take the rest of the disk.')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('checkbox', { name: '/var/lib/docker takes the rest' }))
    expect(size('/var/lib/docker size in GiB')).toHaveValue('')
    fireEvent.change(size('/var/lib/docker size in GiB'), { target: { value: '100' } })
    fireEvent.click(nextButton())
    const storage = wizard().customizationSettings.kuraStorage as { volumes: unknown[] }
    expect(storage.volumes).toContainEqual({ mount: '/home' })
    expect(storage.volumes).toContainEqual({ mount: '/var/lib/docker', sizeGiB: 100 })
  })

  it('warns when the selected drive is too small', async () => {
    answer({})
    setDst('/dev/disk4', 32_000_000_000)
    await renderStep(DiskLayoutStep, kura)
    expect(screen.queryByText(/is smaller than this layout needs/)).toBeNull()
    fireEvent.click(radio(/Server/))
    expect(
      screen.getByText(/The selected drive \(29.8 GB\) is smaller than this layout needs/),
    ).toBeInTheDocument()
  })

  it('restores the saved layout', async () => {
    answer({
      getSavedCustomisationSettings: () => ({
        kuraDiskLayout: 'custom',
        kuraStorage: { rootGiB: 24, tmpGiB: 2, volumes: [{ mount: '/srv' }] },
      }),
    })
    await renderStep(DiskLayoutStep, kura)
    expect(radio(/Custom/)).toBeChecked()
    expect(size('Root size in GiB')).toHaveValue('24')
    expect(screen.getByRole('checkbox', { name: '/srv takes the rest' })).toBeChecked()
  })
})
