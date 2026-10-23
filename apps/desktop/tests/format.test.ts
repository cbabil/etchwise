import { formatSize } from '@/format'

// Same cases as imager-core::format (upstream ImageWriter::formatSize).
describe('formatSize', () => {
  it.each([
    [512, 1, '512 B'],
    [1536, 1, '1.5 KB'],
    [1024 * 1024, 1, '1 MB'],
    [1364720984, 1, '1.3 GB'],
    [6744440832, 0, '6 GB'],
    [1024 * 1024 * 1024 - 1, 1, '1024 MB'],
    [32017047552, 2, '29.82 GB'],
    [2 ** 41, 1, '2 TB'],
    [1075, 2, '1.05 KB'],
  ])('formats %d with %d decimals as %s', (bytes, decimals, expected) => {
    expect(formatSize(bytes, decimals)).toBe(expected)
  })
})
