import { cardGridClassName } from "../styles/uiClasses";
import Skeleton from "./Skeleton";

const PLACEHOLDER_CARDS = Array.from({ length: 8 }, (_, index) => index);

export default function CardGridSkeleton() {
  return (
    <div role="status" aria-label="読み込み中" className={cardGridClassName}>
      {PLACEHOLDER_CARDS.map((index) => (
        <div
          key={index}
          className="overflow-hidden rounded-2xl bg-white shadow-md ring-1 ring-black/5 dark:bg-gray-800 dark:ring-white/5"
        >
          <Skeleton
            className="aspect-[3/4] w-full rounded-none"
            variant="rectangular"
          />
          <div className="flex flex-col gap-2 p-3 md:gap-3 md:p-4">
            <Skeleton className="h-4 w-3/4" />
            <Skeleton className="h-4 w-1/2 rounded-full" />
          </div>
        </div>
      ))}
    </div>
  );
}
