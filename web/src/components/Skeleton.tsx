import { cn } from "../utils/cn";

interface SkeletonProps {
  className?: string;
  variant?: "text" | "circular" | "rectangular";
}

export default function Skeleton({
  className,
  variant = "text",
}: SkeletonProps) {
  return (
    <div
      className={cn(
        "animate-pulse bg-gray-200 dark:bg-gray-700",
        {
          "rounded-md": variant === "text" || variant === "rectangular",
          "rounded-full": variant === "circular",
        },
        className,
      )}
    />
  );
}
