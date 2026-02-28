import { useEffect, useMemo, useState } from "react";

export function useResponsiveColumns(
  breakpoints: { [minWidth: number]: number },
  defaultColumns: number = 2,
): number {
  // Keep the sorted array in a useMemo so it doesn't change reference on every render
  const sortedBreakpoints = useMemo(() => {
    return Object.keys(breakpoints)
      .map(Number)
      .sort((a, b) => b - a);
  }, [breakpoints]);

  const [columns, setColumns] = useState(() => {
    if (typeof window === "undefined") {
      return defaultColumns;
    }
    for (const breakpoint of sortedBreakpoints) {
      if (window.innerWidth >= breakpoint) {
        return breakpoints[breakpoint];
      }
    }
    return defaultColumns;
  });

  useEffect(() => {
    const updateColumns = () => {
      let newColumns = defaultColumns;
      for (const breakpoint of sortedBreakpoints) {
        if (window.innerWidth >= breakpoint) {
          newColumns = breakpoints[breakpoint];
          break;
        }
      }
      setColumns(newColumns);
    };

    window.addEventListener("resize", updateColumns);
    return () => window.removeEventListener("resize", updateColumns);
  }, [breakpoints, defaultColumns, sortedBreakpoints]);

  return columns;
}
