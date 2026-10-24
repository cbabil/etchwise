/**
 * Etchwise's own translations, layered over the upstream catalogues in
 * `locales/` (which `scripts/convert-translations.py` regenerates from
 * Raspberry Pi Imager and would overwrite). Same shape as a catalogue:
 * {language: {context: {source: translation}}}. Wording follows each
 * catalogue's existing style (formal or informal address, verb form); every
 * source here needs an entry for every non-English language (tested).
 */

export type Catalogue = Record<string, Record<string, string>>

export const OVERLAY: Readonly<Record<string, Catalogue>> = {
  bn: { DeviceSelectionStep: { 'Select your device': 'আপনার ডিভাইস নির্বাচন করুন' } },
  ca: { DeviceSelectionStep: { 'Select your device': 'Seleccioneu el vostre dispositiu' } },
  cs: { DeviceSelectionStep: { 'Select your device': 'Výběr zařízení' } },
  de: { DeviceSelectionStep: { 'Select your device': 'Wählen Sie Ihr Gerät aus' } },
  es: { DeviceSelectionStep: { 'Select your device': 'Seleccione su dispositivo' } },
  fr: { DeviceSelectionStep: { 'Select your device': 'Sélectionner votre appareil' } },
  ga: { DeviceSelectionStep: { 'Select your device': 'Roghnaigh do ghléas' } },
  he: { DeviceSelectionStep: { 'Select your device': 'נא לבחור את ההתקן שלך' } },
  hu: { DeviceSelectionStep: { 'Select your device': 'Válassza ki az eszközét' } },
  it: { DeviceSelectionStep: { 'Select your device': 'Seleziona il tuo dispositivo' } },
  ja: { DeviceSelectionStep: { 'Select your device': 'デバイスを選ぶ' } },
  ka: { DeviceSelectionStep: { 'Select your device': 'აირჩიეთ თქვენი მოწყობილობა' } },
  ko: { DeviceSelectionStep: { 'Select your device': '장치를 선택하세요' } },
  nl: { DeviceSelectionStep: { 'Select your device': 'Selecteer je apparaat' } },
  pl: { DeviceSelectionStep: { 'Select your device': 'Wybierz swoje urządzenie' } },
  pt: { DeviceSelectionStep: { 'Select your device': 'Selecionar o dispositivo' } },
  'pt-BR': { DeviceSelectionStep: { 'Select your device': 'Selecionar o dispositivo' } },
  ro: { DeviceSelectionStep: { 'Select your device': 'Selectează dispozitivul' } },
  ru: { DeviceSelectionStep: { 'Select your device': 'Выберите ваше устройство' } },
  sk: { DeviceSelectionStep: { 'Select your device': 'Vyberte svoje zariadenie' } },
  sl: { DeviceSelectionStep: { 'Select your device': 'Izberite svojo napravo' } },
  tr: { DeviceSelectionStep: { 'Select your device': 'Cihazınızı seçin' } },
  uk: { DeviceSelectionStep: { 'Select your device': 'Обрати пристрій' } },
  zh: { DeviceSelectionStep: { 'Select your device': '选择你的设备' } },
  'zh-TW': { DeviceSelectionStep: { 'Select your device': '選擇您的裝置' } },
}

/** `catalogue` with the overlay's entries for `language` added per context. */
export function withOverlay(language: string, catalogue: Catalogue): Catalogue {
  const extra = OVERLAY[language]
  if (!extra) return catalogue
  const merged: Catalogue = { ...catalogue }
  for (const [context, entries] of Object.entries(extra)) {
    merged[context] = { ...catalogue[context], ...entries }
  }
  return merged
}
