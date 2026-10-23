import { render } from '@testing-library/react'

import { styledText } from '@/wizard/dialogs/common/ConfirmDialog'
import { stripEmoji } from '@/wizard/dialogs/common/dialogParts'

describe('dialog text helpers', () => {
  it('renders <b> runs bold and drops other tags', () => {
    const { container } = render(<p>{styledText('a <b>bold</b> <i>c</i>')}</p>)
    expect(container.querySelector('b')?.textContent).toBe('bold')
    expect(container.textContent).toBe('a bold c')
  })

  it('strips emoji prefixes', () => {
    expect(stripEmoji(`${String.fromCodePoint(0x26a0, 0xfe0f)} Careful`)).toBe('Careful')
  })
})
