import * as Dialog from '@radix-ui/react-dialog'
import { AnimatePresence, motion } from 'framer-motion'
import { X } from 'lucide-react'
import type { Config } from '../App'

interface AttributionModalProps {
    isOpen: boolean
    onClose: () => void
    config: Config | null
}

export default function AttributionModal({ isOpen, onClose, config }: AttributionModalProps) {
    if (!config?.attribution?.tmdb) return null

    const { tmdb } = config.attribution

    return (
        <Dialog.Root open={isOpen} onOpenChange={onClose}>
            <AnimatePresence>
                {isOpen && (
                    <Dialog.Portal forceMount>
                        <Dialog.Overlay asChild>
                            <motion.div
                                initial={{ opacity: 0 }}
                                animate={{ opacity: 1 }}
                                exit={{ opacity: 0 }}
                                className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50"
                            />
                        </Dialog.Overlay>
                        <Dialog.Content asChild>
                            <motion.div
                                initial={{ opacity: 0, scale: 0.95 }}
                                animate={{ opacity: 1, scale: 1 }}
                                exit={{ opacity: 0, scale: 0.95 }}
                                transition={{ duration: 0.2 }}
                                className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-md bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 z-50 border border-gray-200 dark:border-gray-700"
                            >
                                <div className="flex justify-between items-center mb-6">
                                    <Dialog.Title className="text-xl font-bold text-gray-900 dark:text-white">
                                        Data Sources
                                    </Dialog.Title>
                                    <Dialog.Close className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 transition-colors">
                                        <X className="w-5 h-5" />
                                    </Dialog.Close>
                                </div>

                                <div className="space-y-8">
                                    {/* TMDb Attribution */}
                                    <div className="flex flex-col items-center text-center space-y-4">
                                        <img
                                            src={tmdb.logo_long}
                                            alt="The Movie Database (TMDb)"
                                            className="h-8 dark:hidden"
                                        />
                                        {/* For dark mode we might need an inverted logo or just use the same one if it works on dark functionality. 
                  TMDb blue logos usually work on both, but let's see. 
                  Actually the 'blue_short' logo works on light/dark. 
                  Let's just use what the backend provides. */}
                                        <img
                                            src={tmdb.logo_long}
                                            alt="The Movie Database (TMDb)"
                                            className="h-8 hidden dark:block"
                                        />

                                        <p className="text-sm text-gray-600 dark:text-gray-300">
                                            This product uses the TMDb API but is not endorsed or certified by TMDb.
                                        </p>
                                    </div>

                                    <div className="border-t border-gray-200 dark:border-gray-700 pt-6">
                                        <h3 className="font-semibold mb-2 text-gray-900 dark:text-white">Other Sources</h3>
                                        <ul className="list-disc list-inside text-sm text-gray-600 dark:text-gray-300 space-y-1">
                                            <li>
                                                <a
                                                    href="https://github.com/bangumi-data/bangumi-data"
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    className="hover:text-blue-500 hover:underline"
                                                >
                                                    Bangumi Data (GitHub)
                                                </a>
                                            </li>
                                            <li>
                                                <a
                                                    href="https://anilist.co"
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    className="hover:text-blue-500 hover:underline"
                                                >
                                                    AniList API
                                                </a>
                                            </li>
                                        </ul>
                                    </div>
                                </div>
                            </motion.div>
                        </Dialog.Content>
                    </Dialog.Portal>
                )}
            </AnimatePresence >
        </Dialog.Root >
    )
}
