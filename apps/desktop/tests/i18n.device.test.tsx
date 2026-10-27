import { translate } from '@/i18n'
import fr from '@/i18n/locales/fr.json'
import hu from '@/i18n/locales/hu.json'
import ko from '@/i18n/locales/ko.json'
import tr from '@/i18n/locales/tr.json'
import zh from '@/i18n/locales/zh.json'
import type { Catalogue } from '@/i18n/overlay'

const NUC = 'Intel NUC'

/** One setup-screen string in `lang`, with a NUC chosen. */
const t = (catalogue: unknown, lang: string, context: string, source: string) =>
  translate(catalogue as Catalogue, context, source, [], { device: NUC, language: lang })

describe('device wording', () => {
  it('names the chosen device on the setup screens a non-Pi device sees', () => {
    expect(
      t({}, 'en', 'UserCustomizationStep', 'Create a user account for your Raspberry Pi'),
    ).toBe('Create a user account for your Intel NUC')
    expect(
      t({}, 'en', 'OSSelectionStep', 'Select an operating system to install on your Raspberry Pi'),
    ).toBe('Select an operating system to install on your Intel NUC')
  })

  it('works in translations, whatever spelling they use', () => {
    for (const [lang, catalogue] of [
      ['fr', fr],
      ['zh', zh],
    ] as const) {
      const text = t(
        catalogue,
        lang,
        'UserCustomizationStep',
        'Create a user account for your Raspberry Pi',
      )
      expect(text, lang).toContain(NUC)
      expect(text, lang).not.toMatch(/Raspberry Pi|树莓派/)
    }
  })

  it('catches the French catalogue\'s "Rasbperry Pi" typo', () => {
    const text = t(
      fr,
      'fr',
      'RemoteAccessStep',
      'Enable secure shell access for remote command-line control of your Raspberry Pi',
    )
    expect(text).toContain(NUC)
    expect(text).not.toMatch(/Ras\w*y Pi/)
  })

  it('says "your device" where the language inflects the name (Turkish, Hungarian, Korean)', () => {
    expect(
      t(tr, 'tr', 'OSSelectionStep', 'Select an operating system to install on your Raspberry Pi'),
    ).toBe('Cihazınıza yüklemek için bir işletim sistemi seçin')
    expect(
      t(
        tr,
        'tr',
        'RemoteAccessStep',
        'Enable secure shell access for remote command-line control of your Raspberry Pi',
      ),
    ).toMatch(/^Cihazınızın uzaktan/)
    expect(
      t(
        hu,
        'hu',
        'HostnameCustomizationStep',
        'A hostname is a unique name that identifies your Raspberry Pi on the network. It should contain only letters, numbers, and hyphens.',
      ),
    ).toContain('azonosítja az eszközt a hálózaton')
    expect(
      t(ko, 'ko', 'UserCustomizationStep', 'Create a user account for your Raspberry Pi'),
    ).toBe('장치 계정을 만듭니다')
  })

  it('takes the device name literally, whatever characters it holds', () => {
    expect(
      translate({}, 'UserCustomizationStep', 'Create a user account for your Raspberry Pi', [], {
        device: 'Box $&',
      }),
    ).toBe('Create a user account for your Box $&')
  })

  it('keeps arguments and product names as they are', () => {
    expect(t({}, 'en', 'OSSelectionStep', 'Install %1 on your Raspberry Pi')).toBe(
      'Install %1 on your Intel NUC',
    )
    expect(
      translate({}, 'OSSelectionStep', 'Install %1 on your Raspberry Pi', ['Raspberry Pi OS'], {
        device: NUC,
      }),
    ).toBe('Install Raspberry Pi OS on your Intel NUC')
    expect(t({}, 'en', 'RemoteAccessStep', 'Use Raspberry Pi Connect')).toBe(
      'Use Raspberry Pi Connect',
    )
  })

  it('leaves no "Raspberry Pi" on the setup screens in any language', () => {
    const catalogues = import.meta.glob<{ default: Catalogue }>('../src/i18n/locales/*.json', {
      eager: true,
    })
    const setup = [
      'OSSelectionStep',
      'HostnameCustomizationStep',
      'LocaleCustomizationStep',
      'UserCustomizationStep',
      'RemoteAccessStep',
    ]
    const leftovers: string[] = []
    for (const [file, { default: catalogue }] of Object.entries(catalogues)) {
      const lang = file.replace(/^.*\/|\.json$/g, '')
      for (const context of setup) {
        for (const source of Object.keys(catalogue[context] ?? {})) {
          const shown = t(catalogue, lang, context, source)
          if (/Raspberry Pi(?! OS| Connect)|Rasbperry Pi|라즈베리 파이|树莓派/.test(shown))
            leftovers.push(`${lang}: ${shown}`)
        }
      }
    }
    expect(leftovers).toEqual([])
  })

  it('leaves the Raspberry Pi wording alone for a Raspberry Pi', () => {
    expect(
      translate({}, 'UserCustomizationStep', 'Create a user account for your Raspberry Pi'),
    ).toBe('Create a user account for your Raspberry Pi')
  })

  it('keeps Raspberry Pi where it names the product, not the device', () => {
    expect(
      t({}, 'en', 'RepositoryDialog', 'Use the official Raspberry Pi operating system repository'),
    ).toBe('Use the official Raspberry Pi operating system repository')
  })

  it('still names the app Etchwise', () => {
    expect(t({}, 'en', 'OSSelectionStep', 'Raspberry Pi Imager lists your Raspberry Pi')).toBe(
      'Etchwise lists your Intel NUC',
    )
  })
})
