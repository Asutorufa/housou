// @vitest-environment jsdom
import { renderHook, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { useItems } from './useItems'

describe('useItems', () => {
  const mockFetch = vi.fn()
  global.fetch = mockFetch

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('should return initial loading state as true', () => {
    const { result } = renderHook(() => useItems('', ''))
    expect(result.current.loading).toBe(true)
    expect(result.current.items).toEqual([])
    expect(result.current.error).toBeNull()
  })

  it('should fetch items when year is provided', async () => {
    const mockData = [{ title: 'Anime 1' }]
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockData,
    })

    const { result } = renderHook(() => useItems('2023', 'all'))

    expect(result.current.loading).toBe(true)

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(result.current.items).toEqual(mockData)
    expect(result.current.error).toBeNull()
    expect(mockFetch).toHaveBeenCalledWith('/api/items?year=2023', expect.any(Object))
  })

  it('should append season to URL if provided', async () => {
    const mockData = [{ title: 'Anime 1' }]
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockData,
    })

    const { result } = renderHook(() => useItems('2023', 'Winter'))

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(mockFetch).toHaveBeenCalledWith('/api/items?year=2023&season=Winter', expect.any(Object))
  })

  it('should handle fetch errors', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
    })

    const { result } = renderHook(() => useItems('2023', 'all'))

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(result.current.error).toBe('Items fetch failed')
    expect(result.current.items).toEqual([])
  })

  it('should handle network errors', async () => {
    mockFetch.mockRejectedValueOnce(new Error('Network error'))

    const { result } = renderHook(() => useItems('2023', 'all'))

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(result.current.error).toBe('Network error')
  })

  it('should abort fetch when unmounted or dependencies change', async () => {
    const abortSpy = vi.fn()
    mockFetch.mockImplementation(() => {
      return new Promise((resolve) => {
        // simulate pending request
        setTimeout(() => {
            resolve({
                ok: true,
                json: async () => [],
            })
        }, 100)
      })
    })

    // Mock AbortController
    const originalAbortController = global.AbortController
    global.AbortController = class {
        signal = { aborted: false } as AbortSignal
        abort = abortSpy
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } as any

    const { unmount } = renderHook(
      ({ year, season }) => useItems(year, season),
      { initialProps: { year: '2023', season: 'all' } }
    )

    // Unmount triggers cleanup
    unmount()
    expect(abortSpy).toHaveBeenCalled()

    // Restore AbortController
    global.AbortController = originalAbortController
  })
})
