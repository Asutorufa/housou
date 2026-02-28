import { useEffect, useState } from "react";

export function useResponsiveColumns(
  breakpoints: { [minWidth: number]: number },
  defaultColumns: number = 2,
): number {
  const [columns, setColumns] = useState(defaultColumns);

  useEffect(() => {
    const updateColumns = () => {
      const width = window.innerWidth;
      // Sort breakpoints from largest to smallest
      const sortedBreakpoints = Object.keys(breakpoints)
        .map(Number)
        .sort((a, b) => b - a);

      let newColumns = defaultColumns;
      for (const breakpoint of sortedBreakpoints) {
        if (width >= breakpoint) {
          newColumns = breakpoints[breakpoint];
          break;
        }
      }
      setColumns(newColumns);
    };

    updateColumns();
    window.addEventListener("resize", updateColumns);
    return () => window.removeEventListener("resize", updateColumns);
  }, [breakpoints, defaultColumns]);

  return columns;
}
