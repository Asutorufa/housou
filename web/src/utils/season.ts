const SEASONS = [
  { value: "Winter", label: "冬", startMonth: 1 },
  { value: "Spring", label: "春", startMonth: 4 },
  { value: "Summer", label: "夏", startMonth: 7 },
  { value: "Autumn", label: "秋", startMonth: 10 },
];

export function getSeasonOptions(
  selectedYear: string,
  currentYear: number,
  currentMonth: number,
) {
  const selYear = parseInt(selectedYear, 10);
  const getLabel = (label: string, startMonth: number) => {
    if (isNaN(selYear)) return label;

    const isFuture =
      selYear > currentYear ||
      (selYear === currentYear && startMonth > currentMonth);
    return isFuture ? `${label} (予定)` : label;
  };

  return [
    { value: "all", label: "全て" },
    ...SEASONS.map((season) => ({
      value: season.value,
      label: getLabel(season.label, season.startMonth),
    })),
  ];
}
