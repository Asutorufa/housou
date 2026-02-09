import { useEffect, useState } from 'react'
import type { AnimeItem } from '../types'

export function useItems(year: string, season: string) {
  const [items, setItems] = useState<AnimeItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!year) return

    const controller = new AbortController()
    setLoading(true)

    async function fetchItems() {
      try {
        let url = `/api/items?year=${year}`
        if (season && season !== 'all') {
          url += `&season=${season}`
        }

        const response = await fetch(url, { signal: controller.signal })
        if (!response.ok) {
          throw new Error('Items fetch failed')
        }

        const data: AnimeItem[] = await response.json()

        if (!controller.signal.aborted) {
          setItems(data)
          setLoading(false)
          setError(null)
        }
      } catch (err) {
        if (err instanceof Error && err.name === 'AbortError') {
          return
        }
        if (!controller.signal.aborted) {
          setError(err instanceof Error ? err.message : String(err))
          setLoading(false)
        }
      }
    }

    fetchItems()

    return () => {
      controller.abort()
    }
  }, [year, season])

  return { items, loading, error }
}
