
import { render, screen } from '@testing-library/react'
import DetailsModal from './DetailsModal'
import { describe, it, expect } from 'vitest'


// Mock anime data
const mockAnime = {
  title: 'Test Anime',
  info: {
    id: '1',
    title: { native: 'Test Anime' },
    coverImage: { large: 'http://example.com/image.jpg' },
    genres: [],
    studios: [],
    characters: [],
    staff: [],
    episodesList: [],
    isFinished: false,
    description: '<script>alert("xss")</script>',
  },
}

const mockItems = [
  {
    title: 'Test Anime',
    type: 'tv',
    lang: 'ja',
    officialSite: '',
    begin: '',
    end: '',
    sites: [],
  },
]

describe('DetailsModal', () => {
  it('renders description safely', () => {
    render(
      <DetailsModal
        isOpen={true}
        onClose={() => {}}
        anime={mockAnime}
        items={mockItems}
      />
    )

    // Find the description element
    const titleElement = screen.getByText('あらすじ')
    expect(titleElement).toBeTruthy()

    const descriptionContainer = titleElement.nextElementSibling
    expect(descriptionContainer).toBeTruthy()

    console.log('innerHTML:', descriptionContainer?.innerHTML)

    expect(descriptionContainer?.innerHTML).not.toContain('<script>')
    expect(descriptionContainer?.textContent).toContain('<script>alert("xss")</script>')
  })
})
