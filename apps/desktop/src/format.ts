/**
 * Upstream `ImageWriter::formatSize`: binary units, rounded to `decimals`
 * fractional digits, with a zero fraction omitted.
 */
export function formatSize(
  bytes: number,
  decimals = 1,
  units = ['B', 'KB', 'MB', 'GB', 'TB'],
): string {
  const steps = [4, 3, 2, 1].map((p) => 1024 ** p)
  const index = steps.findIndex((size) => bytes >= size)
  const unit = index === -1 ? 1 : (steps[index] as number)
  const name = index === -1 ? units[0] : units[4 - index]
  const scale = 10 ** Math.max(0, decimals)
  let whole = Math.floor(bytes / unit)
  let frac = Math.floor(((bytes % unit) * scale + Math.floor(unit / 2)) / unit)
  if (frac >= scale) {
    whole += 1
    frac -= scale
  }
  if (frac === 0) return `${whole} ${name}`
  return `${whole}.${String(frac).padStart(decimals, '0')} ${name}`
}
