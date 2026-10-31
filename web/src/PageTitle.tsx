import {useLayoutEffect, useRef} from 'react'

type PageTitleProps = {
  value: string
  onChange: (value: string) => void
  onCommit: (value: string) => void
  placeholder?: string
}

export function PageTitle({value, onChange, onCommit, placeholder = 'Untitled'}: PageTitleProps) {
  const ref = useRef<HTMLHeadingElement>(null)

  useLayoutEffect(() => {
    const el = ref.current
    if (!el) return
    // Sync only when the DOM disagrees with state. The extra clause clears the
    // browser's placeholder <br> left behind by an emptied editable -- the
    // textContent comparison alone cannot see it.
    if (el.textContent !== value || (value === '' && el.childNodes.length > 0)) {
      el.textContent = value
    }
  }, [value])

  useLayoutEffect(() => {
    const el = ref.current
    if (!el) return
    el.focus()
    const selection = window.getSelection()
    if (!selection) return
    const range = document.createRange()
    range.selectNodeContents(el)
    range.collapse(false)
    selection.removeAllRanges()
    selection.addRange(range)
  }, [])

  return (
    <h1
      ref={ref}
      contentEditable="plaintext-only"
      suppressContentEditableWarning
      autoCapitalize="off"
      autoCorrect="off"
      spellCheck={false}
      translate="no"
      data-empty={value === '' ? 'true' : 'false'}
      data-placeholder={placeholder}
      onInput={e => onChange(e.currentTarget.textContent ?? '')}
      onBlur={e => onCommit(e.currentTarget.textContent?.replace(/\s+/g, ' ').trim() ?? '')}
      onKeyDown={e => {
        if (e.key === 'Enter' && !e.nativeEvent.isComposing) {
          e.preventDefault()
          e.currentTarget.blur()
        }
      }}
    />
  )
}

export default PageTitle
