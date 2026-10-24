import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'

import { Step } from '@/wizard/stepIds'
import { stepComponents } from '@/wizard/steps'
import { WizardContainer } from '@/wizard/WizardContainer'
import { type WizardApi, WizardProvider } from '@/wizard/WizardContext'

import { Capture } from './capture'

const next = () => fireEvent.click(screen.getByRole('button', { name: /^Next/ }))
/** Lets a row selection's backend calls settle, as a user would, then Next. */
const nextWhenEnabled = async () => {
  await act(() => new Promise((resolve) => setTimeout(resolve, 300)))
  await waitFor(() => expect(screen.getByRole('button', { name: /^Next/ })).toBeEnabled())
  next()
}
const title = (name: string) => screen.findByRole('heading', { level: 1, name })

function renderWizard(showLanguageSelection = false) {
  return render(
    <WizardProvider showLanguageSelection={showLanguageSelection}>
      <WizardContainer />
    </WizardProvider>,
  )
}

describe('WizardContainer', () => {
  it('registers every step index', () => {
    const indices = Object.keys(stepComponents)
      .map(Number)
      .sort((a, b) => a - b)
    expect(indices).toEqual([-1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12])
  })

  it('walks Language, Device, OS and Storage to Writing with a fade between steps', async () => {
    renderWizard(true)
    await title('Welcome')
    next()
    expect(screen.getByRole('main')).toHaveStyle({ opacity: '0' })
    await title('Select your device')
    expect(screen.queryByRole('button', { name: /^Back/ })).toBeNull()
    fireEvent.click(await screen.findByText('Raspberry Pi 5'))
    await nextWhenEnabled()
    await title('Choose operating system')
    fireEvent.click(await screen.findByText('Raspberry Pi OS (64-bit)'))
    await nextWhenEnabled()
    await title('Select your storage device')
    fireEvent.click(screen.getByRole('button', { name: /^Back/ }))
    await title('Choose operating system')
    // Upstream rebuilds the step (StackView clear + push): the list starts
    // without a current row, so the OS is picked again.
    fireEvent.click(await screen.findByText('Raspberry Pi OS (64-bit)'))
    await nextWhenEnabled()
    await title('Select your storage device')
    fireEvent.click(await screen.findByText('Apple SDXC Reader Media'))
    await nextWhenEnabled()
    await title('Customisation: Choose hostname')
    fireEvent.click(screen.getByRole('button', { name: /^Skip customisation/ }))
    await title('Write image')
  })

  it('opens App Options from the sidebar and closes it', async () => {
    renderWizard()
    await title('Select your device')
    fireEvent.click(screen.getByRole('button', { name: /^App Options/ }))
    const dialog = screen.getByRole('dialog', { name: 'App Options' })
    fireEvent.click(screen.getByRole('button', { name: /^Save(,|$)/ }))
    await waitFor(() => expect(dialog).not.toBeInTheDocument())
    fireEvent.click(screen.getByRole('button', { name: /^App Options/ }))
    fireEvent.click(screen.getByRole('button', { name: /^Cancel/ }))
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
  })
})

let wizard: WizardApi
const keep = (w: WizardApi) => {
  wizard = w
}

describe('customisation steps in the container', () => {
  it.each([
    [Step.Hostname, 'Customisation: Choose hostname'],
    [Step.Locale, 'Customisation: Localisation'],
    [Step.User, 'Customisation: Choose username'],
    [Step.Wifi, 'Customisation: Choose Wi\u2011Fi'],
    [Step.RemoteAccess, 'Customisation: SSH authentication'],
    [Step.SecureBoot, 'Customisation: Secure Boot'],
    [Step.PiConnect, 'Customisation: Raspberry Pi Connect'],
  ])('step %i shows its title and skips customisation to Writing', async (step, name) => {
    const Component = stepComponents[step]!
    render(
      <WizardProvider showLanguageSelection={false}>
        <Capture onWizard={keep} />
        <Component />
      </WizardProvider>,
    )
    await waitFor(() => expect(wizard.ready).toBe(true))
    expect(await screen.findByRole('heading', { level: 1, name })).toBeInTheDocument()
    act(() => wizard.jumpTo(step))
    act(() => wizard.update({ sshEnabled: true, ifI2cEnabled: true }))
    fireEvent.click(screen.getByRole('button', { name: /^Skip customisation/ }))
    await waitFor(() =>
      expect(wizard).toMatchObject({ currentStep: Step.Writing, sshEnabled: false }),
    )
  })
})
