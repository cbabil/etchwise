/**
 * In-memory stand-in for the backend so the UI runs in a plain browser
 * (`bun run dev`) and in unit tests. Data is small but shaped like the
 * real backend's answers.
 */
import type { ImagerEvent } from './imager'
import { mockOsList } from './mockData'

type Handler = (event: ImagerEvent) => void

const handlers = new Set<Handler>()
const settings: Record<string, unknown> = {}
let customisation: Record<string, unknown> = {}
let source: Record<string, unknown> = {}
let debugOptions: Record<string, unknown> = { asyncQueueDepth: 4 }

const answers: Record<string, (args: unknown[]) => unknown> = {
  getBoolSetting: ([key]) =>
    settings[String(key)] ?? ['eject', 'check_version'].includes(String(key)),
  getStringSetting: ([key]) => String(settings[String(key)] ?? ''),
  setSetting: ([key, value]) => void (settings[String(key)] = value),
  getSavedCustomisationSettings: () => ({ ...customisation }),
  setSavedCustomisationSettings: ([map]) => void (customisation = { ...(map as object) }),
  setPersistedCustomisationSetting: ([key, value]) =>
    void (customisation = { ...customisation, [String(key)]: value }),
  removePersistedCustomisationSetting: ([key]) => {
    customisation = Object.fromEntries(
      Object.entries(customisation).filter(([k]) => k !== String(key)),
    )
  },
  clearSavedCustomisationSettings: () => void (customisation = {}),
  constantVersion: () => 'v1.0.0',
  getHWList: () => mockOsList.hw,
  getOSList: () => mockOsList.os,
  getSublist: () => [],
  isOsListUnavailable: () => false,
  getDriveList: () => mockOsList.drives,
  getTimezoneList: () => ['Europe/London', 'Europe/Paris', ''],
  getKeymapLayoutList: () => ['gb', 'us', 'fr'],
  getCapitalCitiesList: () => ['London (United Kingdom)', 'Paris (France)'],
  getLocaleDataForCapital: () => ({ timezone: 'Europe/London', keyboard: 'gb', countryCode: 'GB' }),
  getTimezone: () => 'Europe/London',
  progress: () => ({
    state: 'Idle',
    downloadNow: 0,
    downloadTotal: 0,
    bytesWritten: 0,
    writeTotal: 0,
    verifyNow: 0,
    verifyTotal: 0,
  }),
  isBeepAvailable: () => true,
  nativeFileDialogAvailable: () => true,
  readyToWrite: () => true,
  setSrc: ([src]) => void (source = { ...(src as object) }),
  imageSupportsCustomization: () => Boolean(source['initFormat']),
  imageSupportsCcRpi: () => source['initFormat'] === 'cloudinit-rpi',
  srcFileName: () =>
    String(source['url'] ?? '')
      .split('/')
      .pop() ?? '',
  getSelectedSourceSize: () => Number(source['extractLen'] ?? 0),
  isExtractSizeKnown: () => true,
  isValidRepoUrl: ([url]) => /^https?:\/\/\S+\.(json|rpi-imager-manifest)$/.test(String(url)),
  osListUrl: () => 'https://downloads.raspberrypi.com/os_list_imagingutility_v4.json',
  customRepo: () => false,
  getDebugOptions: () => ({ ...debugOptions }),
  setDebugOptions: ([options]) => void (debugOptions = { ...(options as object) }),
  writeState: () => 'Idle',
  getSSID: () => '',
  getPSKForSSID: () => '',
  crypt: ([password]) => `$y$jB5$mock$${String(password).length}`,
  pbkdf2: () => 'f'.repeat(64),
  getRuntimeConnectToken: () => '',
  checkSWCapability: () => false,
  checkHWCapability: () => false,
  checkHWAndSWCapability: () => false,
  isCached: () => false,
  hasWindowDecorations: () => true,
}

export function mockCall(method: string, args: unknown[]): unknown {
  return answers[method]?.(args) ?? null
}

export function mockListen(handler: Handler): () => void {
  handlers.add(handler)
  return () => handlers.delete(handler)
}

/** Test helper: deliver a fake backend signal. */
export function mockEmit(event: ImagerEvent): void {
  handlers.forEach((h) => h(event))
}
