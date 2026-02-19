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
    { value: "Winter", label: getLabel("冬", 1) },
    { value: "Spring", label: getLabel("春", 4) },
    { value: "Summer", label: getLabel("夏", 7) },
    { value: "Autumn", label: getLabel("秋", 10) },
  ];
}
