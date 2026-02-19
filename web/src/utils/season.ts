export const SEASONS = [
  { value: "Winter", label: "冬" },
  { value: "Spring", label: "春" },
  { value: "Summer", label: "夏" },
  { value: "Autumn", label: "秋" },
] as const;

export function getSeasonLabel(
  seasonValue: string,
  baseLabel: string,
  selectedYear: string | number,
  currentDate: Date = new Date(),
): string {
  const currentYear = currentDate.getFullYear();
  const currentMonth = currentDate.getMonth();
  const currentSeasonIndex = Math.floor(currentMonth / 3);

  const yearNum = Number(selectedYear);

  if (isNaN(yearNum)) {
    return baseLabel;
  }

  const seasonIndex = SEASONS.findIndex((s) => s.value === seasonValue);

  if (seasonIndex === -1) {
    return baseLabel;
  }

  if (
    yearNum > currentYear ||
    (yearNum === currentYear && seasonIndex > currentSeasonIndex)
  ) {
    return `${baseLabel} (予定)`;
  }

  return baseLabel;
}
