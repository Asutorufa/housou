import { Suspense, useState, type ReactNode } from "react";

/** Load on first open, then let the dialog manage its exit animation. */
export default function DeferredDialog({
  open,
  children,
}: {
  open: boolean;
  children: ReactNode;
}) {
  const [hasOpened, setHasOpened] = useState(false);
  if (open && !hasOpened) setHasOpened(true);

  return open || hasOpened ? (
    <Suspense fallback={null}>{children}</Suspense>
  ) : null;
}
