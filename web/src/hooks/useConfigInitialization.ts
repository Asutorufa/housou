import { useEffect, useState, type Dispatch, type SetStateAction } from "react";
import type { Config, Selections } from "../types";

export function useConfigInitialization(
  setSelections: Dispatch<SetStateAction<Selections>>,
) {
  const [config, setConfig] = useState<Config | null>(null);
  const [initError, setInitError] = useState<string | null>(null);
  const [initLoading, setInitLoading] = useState(true);

  // Initialization
  useEffect(() => {
    const controller = new AbortController();

    async function init() {
      try {
        const response = await fetch(`/api/config?v=${new Date().getTime()}`, {
          signal: controller.signal,
        });
        if (!response.ok) throw new Error("Config fetch failed");
        const data: Config = await response.json();
        setConfig(data);

        // Validate or set defaults
        setSelections((prev) => {
          let { year, season } = prev;
          const { site, status } = prev;
          const isYearValid = year && data.years.includes(parseInt(year));

          if (!isYearValid) {
            // Apply defaults
            const currentYear = new Date().getFullYear().toString();
            if (data.years.includes(parseInt(currentYear))) {
              year = currentYear;
            } else if (data.years.length > 0) {
              year = data.years[data.years.length - 1].toString();
            } else {
              year = "";
            }

            // Get current season
            const seasons = ["Winter", "Spring", "Summer", "Autumn"];
            season = seasons[Math.floor(new Date().getMonth() / 3)];
          }

          return { year, season, site, status: status || "all" };
        });
      } catch (err) {
        if (err instanceof Error && err.name === "AbortError") return;
        setInitError(err instanceof Error ? err.message : String(err));
      } finally {
        if (!controller.signal.aborted) {
          setInitLoading(false);
        }
      }
    }
    init();

    return () => {
      controller.abort();
    };
  }, [setSelections]);

  return { config, initError, initLoading };
}
