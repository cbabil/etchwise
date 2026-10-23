/** Sample backend answers for the mock (shapes match the Rust side). */
export const mockOsList = {
  hw: {
    currentIndex: -1,
    devices: [
      {
        name: 'Raspberry Pi 5',
        tags: ['pi5-64bit'],
        capabilities: [],
        icon: '',
        description: 'Raspberry Pi 5, 500 / 500+, and Compute Module 5',
        matching_type: 'exclusive',
        architecture: 'arm64',
      },
      {
        name: 'No filtering',
        tags: [],
        capabilities: [],
        icon: '',
        description: 'Show every possible option',
        matching_type: 'inclusive',
        architecture: '',
      },
    ],
  },
  os: [
    {
      name: 'Raspberry Pi OS (64-bit)',
      description: 'A port of Debian Trixie with the Raspberry Pi Desktop',
      recommended: true,
      url: 'https://downloads.raspberrypi.com/raspios_arm64/images/x.img.xz',
      icon: '',
      extract_size: 6744440832,
      image_download_size: 1364720984,
      extract_sha256: 'aa',
      release_date: '2026-10-06',
      init_format: 'cloudinit-rpi',
      devices: ['pi5-64bit'],
      capabilities: ['rpi_connect'],
      subitems_json: '',
    },
    {
      name: 'Erase',
      description: 'Format card as FAT32',
      icon: 'icons/erase.png',
      url: 'internal://format',
      subitems_json: '',
    },
    {
      name: 'Use custom',
      description: 'Select a custom .img from your computer',
      icon: 'icons/use_custom.png',
      url: 'internal://custom',
      subitems_json: '',
    },
  ],
  drives: {
    error: '',
    drives: [
      {
        key: '/dev/disk4:31914983424',
        device: '/dev/disk4',
        description: 'Apple SDXC Reader Media',
        size: 31914983424,
        isUsb: false,
        isScsi: false,
        isReadOnly: false,
        isSystem: false,
        mountpoints: ['/Volumes/bootfs'],
        childDevices: [],
      },
    ],
  },
}
