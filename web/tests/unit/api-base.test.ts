import { describe, expect, it } from 'vitest'
import { normalizeApiBase } from '../../utils/api-base'

describe('normalizeApiBase', () => {
  it('uses the current origin for the default relative API path', () => {
    expect(normalizeApiBase()).toBe('')
    expect(normalizeApiBase('/')).toBe('')
  })

  it('removes only the trailing slash from configured origins', () => {
    expect(normalizeApiBase('http://127.0.0.1:3000/')).toBe('http://127.0.0.1:3000')
    expect(normalizeApiBase('http://127.0.0.1:3000')).toBe('http://127.0.0.1:3000')
  })
})
