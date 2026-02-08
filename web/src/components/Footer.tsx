import { Github } from 'lucide-react'

interface FooterProps {
    onOpenAttribution: () => void
}

export default function Footer({ onOpenAttribution }: FooterProps) {
    return (
        <footer className="mt-12 pb-8 text-center text-sm text-gray-500 dark:text-gray-400">
            <div className="flex flex-col items-center gap-4">
                <div className="flex items-center gap-4">
                    <a
                        href="https://github.com/Asutorufa/housou"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="hover:text-gray-900 dark:hover:text-gray-100 transition-colors"
                    >
                        <Github className="w-5 h-5" />
                    </a>
                </div>

                <div className="flex flex-col sm:flex-row items-center gap-2">
                    <p>© {new Date().getFullYear()} Housou. All rights reserved.</p>
                    <span className="hidden sm:inline">•</span>
                    <button
                        onClick={onOpenAttribution}
                        className="hover:text-blue-500 hover:underline transition-colors"
                    >
                        Data Sources & Attribution
                    </button>
                </div>
            </div>
        </footer>
    )
}
