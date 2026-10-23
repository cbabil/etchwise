/** OS list rows shared by the OS step tests. */
import type { OsEntry } from '@/wizard/steps/os/types'

export const piOs: OsEntry = {
  name: 'Raspberry Pi OS (64-bit)',
  description: 'A port of Debian',
  recommended: true,
  url: 'https://downloads/x.img.xz',
  icon: 'https://downloads/icon.png',
  extract_size: 6000,
  image_download_size: 1364720984,
  extract_sha256: 'aa',
  release_date: '2026-10-06',
  init_format: 'cloudinit-rpi',
  capabilities: ['rpi_connect'],
  subitems_json: '',
}

export const lite: OsEntry = {
  name: 'Lite',
  description: 'No desktop',
  url: 'https://downloads/lite.img.xz',
  image_download_size: 512 * 1024 * 1024,
  extract_sha256: 'bb',
  release_date: '',
  subitems_json: '',
}

export const group: OsEntry = {
  name: 'Other',
  description: 'More images',
  icon: 'icons/cat_raspberry_pi_os.png',
  url: '',
  subitems_json: '[{"name":"Lite"}]',
}

export const erase: OsEntry = {
  name: 'Erase',
  description: 'Format card as FAT32',
  icon: 'icons/erase.png',
  url: 'internal://format',
}

export const custom: OsEntry = {
  name: 'Use custom',
  description: 'Select a custom .img from your computer',
  icon: 'icons/use_custom.png',
  url: 'internal://custom',
}

export const mainList = [piOs, group, erase, custom]
