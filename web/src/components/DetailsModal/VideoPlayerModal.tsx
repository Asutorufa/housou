import * as Dialog from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";

interface VideoPlayerModalProps {
  isOpen: boolean;
  onClose: () => void;
  videoKey: string | null;
}

export default function VideoPlayerModal({
  isOpen,
  onClose,
  videoKey,
}: VideoPlayerModalProps) {
  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <AnimatePresence>
        {isOpen && videoKey && (
          <Dialog.Portal forceMount>
            <Dialog.Overlay asChild>
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 z-[200] bg-black/80 backdrop-blur-md"
              />
            </Dialog.Overlay>
            <Dialog.Content asChild>
              <div
                className="fixed inset-0 z-[210] flex items-center justify-center p-4 sm:p-6"
                onClick={onClose}
              >
                <motion.div
                  initial={{ opacity: 0, scale: 0.95 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.95 }}
                  transition={{ duration: 0.2 }}
                  className="relative w-full max-w-5xl overflow-hidden rounded-2xl bg-black shadow-2xl ring-1 ring-white/10"
                  onClick={(e) => e.stopPropagation()}
                >
                  <div className="relative aspect-video w-full">
                    <iframe
                      src={`https://www.youtube.com/embed/${videoKey}?autoplay=1&rel=0`}
                      title="YouTube video player"
                      className="absolute inset-0 h-full w-full border-0"
                      allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                      allowFullScreen
                    />
                  </div>
                  <Dialog.Close asChild>
                    <button
                      className="absolute -top-12 right-0 z-[220] rounded-full bg-white/10 p-2 text-white hover:bg-white/20 focus:outline-none sm:top-4 sm:right-4"
                      onClick={onClose}
                    >
                      <X size={24} />
                    </button>
                  </Dialog.Close>
                </motion.div>
              </div>
            </Dialog.Content>
          </Dialog.Portal>
        )}
      </AnimatePresence>
    </Dialog.Root>
  );
}
