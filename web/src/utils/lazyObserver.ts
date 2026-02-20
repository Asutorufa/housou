let intersectionObserver: IntersectionObserver | null = null;
const observerCallbacks = new WeakMap<Element, () => void>();

export function getLazyObserver() {
  if (typeof window === "undefined") return null;
  if (!intersectionObserver) {
    intersectionObserver = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const callback = observerCallbacks.get(entry.target);
            if (callback) {
              callback();
              intersectionObserver?.unobserve(entry.target);
              observerCallbacks.delete(entry.target);
            }
          }
        });
      },
      { rootMargin: "300px" },
    );
  }
  return intersectionObserver;
}

export function observeLazy(element: Element, callback: () => void) {
  const observer = getLazyObserver();
  if (observer) {
    observerCallbacks.set(element, callback);
    observer.observe(element);
  }
}

export function unobserveLazy(element: Element) {
  const observer = getLazyObserver();
  if (observer) {
    observer.unobserve(element);
    observerCallbacks.delete(element);
  }
}
