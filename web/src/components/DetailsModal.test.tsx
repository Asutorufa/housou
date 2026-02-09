
import { render, screen } from '@testing-library/react'
import DetailsModal from './DetailsModal'
import { describe, it, expect } from 'vitest'

// Mock ResizeObserver
window.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

// Mock PointerEvent
if (!window.PointerEvent) {
  class PointerEvent extends MouseEvent {
    public height: number;
    public isPrimary: boolean;
    public pointerId: number;
    public pointerType: string;
    public pressure: number;
    public tangentialPressure: number;
    public tiltX: number;
    public tiltY: number;
    public twist: number;
    public width: number;

    constructor(type: string, params: PointerEventInit = {}) {
      super(type, params);
      this.pointerId = params.pointerId || 0;
      this.width = params.width || 0;
      this.height = params.height || 0;
      this.pressure = params.pressure || 0;
      this.tangentialPressure = params.tangentialPressure || 0;
      this.tiltX = params.tiltX || 0;
      this.tiltY = params.tiltY || 0;
      this.twist = params.twist || 0;
      this.pointerType = params.pointerType || "";
      this.isPrimary = params.isPrimary || false;
    }
  }
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  window.PointerEvent = PointerEvent as any;
}


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
