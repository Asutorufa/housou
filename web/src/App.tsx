import { useEffect, useMemo, useState } from 'react'
import AttributionModal from './components/AttributionModal'
import DetailsModal from './components/DetailsModal'
import Footer from './components/Footer'
import Header from './components/Header'
import TabbedGrid from './components/TabbedGrid'
import type { AnimeItem, Config, UnifiedMetadata } from './types'

export default function App() {
  const [config, setConfig] = useState<Config | null>(null)
  const [items, setItems] = useState<AnimeItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const [selectedYear, setSelectedYear] = useState<string>('')
  const [selectedSeason, setSelectedSeason] = useState<string>('all')
  const [selectedSite, setSelectedSite] = useState<string>('all')
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedAnime, setSelectedAnime] = useState<{ title: string; info: UnifiedMetadata | null } | null>(null)
  const [isAttributionOpen, setIsAttributionOpen] = useState(false)

  // Tabs state (Lifted from TabbedGrid)
  const [activeTab, setActiveTab] = useState(new Date().getDay().toString())
  const [direction, setDirection] = useState(0)

  const handleTabChange = (newTab: string) => {
    const prevIndex = parseInt(activeTab)
    const nextIndex = parseInt(newTab)
    const total = 8 // WEEKDAYS.length

    // Calculate shortest direction with wrap-around
    let diff = nextIndex - prevIndex
    if (Math.abs(diff) > total / 2) {
      diff = diff > 0 ? diff - total : diff + total
    }

    setDirection(diff > 0 ? 1 : -1)
    setActiveTab(newTab)
  }

  // Initialization
  useEffect(() => {
    async function init() {
      try {
        const response = await fetch(`/api/config?v=${new Date().getTime()}`)
        if (!response.ok) throw new Error('Config fetch failed')
        const data: Config = await response.json()
        setConfig(data)

        // Load saved selections
        const saved = localStorage.getItem('housou_selections')
        if (saved) {
          const { year, season, site } = JSON.parse(saved)
          if (data.years.includes(parseInt(year))) setSelectedYear(year)
          setSelectedSeason(season || 'all')
          setSelectedSite(site || 'all')
        } else {
          const currentYear = new Date().getFullYear().toString()
          if (data.years.includes(parseInt(currentYear))) {
            setSelectedYear(currentYear)
          } else if (data.years.length > 0) {
            setSelectedYear(data.years[data.years.length - 1].toString())
          }

          // Get current season
          const month = new Date().getMonth() + 1
          if (month >= 1 && month <= 3) setSelectedSeason('Winter')
          else if (month >= 4 && month <= 6) setSelectedSeason('Spring')
          else if (month >= 7 && month <= 9) setSelectedSeason('Summer')
          else setSelectedSeason('Autumn')
        }
      } catch (err: any) {
        setError(err.message)
      }
    }
    init()
  }, [])

  // Save selections
  useEffect(() => {
    if (selectedYear) {
      localStorage.setItem('housou_selections', JSON.stringify({
        year: selectedYear,
        season: selectedSeason,
        site: selectedSite
      }))
    }
  }, [selectedYear, selectedSeason, selectedSite])

  // Fetch items
  useEffect(() => {
    if (!selectedYear) return

    async function fetchItems() {
      setLoading(true)
      try {
        let url = `/api/items?year=${selectedYear}`
        if (selectedSeason && selectedSeason !== 'all') url += `&season=${selectedSeason}`
        const response = await fetch(url)
        if (!response.ok) throw new Error('Items fetch failed')
        const data = await response.json()
        setItems(data)
      } catch (err: any) {
        setError(err.message)
      } finally {
        setLoading(false)
      }
    }
    fetchItems()
  }, [selectedYear, selectedSeason])

  // Filter items
  const filteredItems = useMemo(() => {
    let filtered = items

    if (selectedSite && selectedSite !== 'all') {
      filtered = filtered.filter(item =>
        item.sites?.some(s => s.site === selectedSite)
      )
    }

    if (searchQuery) {
      const query = searchQuery.toLowerCase()
      filtered = filtered.filter(item => {
        if (item.title.toLowerCase().includes(query)) return true
        if (item.titleTranslate) {
          return Object.values(item.titleTranslate).some(ts =>
            ts?.some(t => t.toLowerCase().includes(query))
          )
        }
        return false
      })
    }

    return filtered
  }, [items, selectedSite, searchQuery])

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-100 dark:bg-gray-900 text-red-500 p-4">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-2">Error</h1>
          <p>{error}</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900 text-gray-900 dark:text-gray-100 transition-colors">
      <Header
        config={config}
        selectedYear={selectedYear}
        setSelectedYear={setSelectedYear}
        selectedSeason={selectedSeason}
        setSelectedSeason={setSelectedSeason}
        selectedSite={selectedSite}
        setSelectedSite={setSelectedSite}
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
        activeTab={activeTab}
        onTabChange={handleTabChange}
      />

      <main className="max-w-7xl mx-auto p-4 sm:p-6 lg:p-8">
        {loading ? (
          <div className="flex justify-center py-12">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
          </div>
        ) : (
          <TabbedGrid
            items={filteredItems}
            siteMeta={config?.site_meta}
            selectedSite={selectedSite}
            onOpenModal={(title: string, info: UnifiedMetadata | null) => setSelectedAnime({ title, info })}
            activeTab={activeTab}
            direction={direction}
            onTabChange={handleTabChange}
          />
        )}
        <Footer onOpenAttribution={() => setIsAttributionOpen(true)} />
      </main>

      <DetailsModal
        isOpen={!!selectedAnime}
        onClose={() => setSelectedAnime(null)}
        anime={selectedAnime}
        items={items}
        siteMeta={config?.site_meta}
      />

      <AttributionModal
        isOpen={isAttributionOpen}
        onClose={() => setIsAttributionOpen(false)}
        config={config}
      />
    </div>
  )
}
