export default function StudioSection({ studios }: { studios?: string[] }) {
  if (!studios || studios.length === 0) return null;

  return (
    <div className="sm:col-span-2">
      <h4 className="mb-2 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
        スタジオ
      </h4>
      <div className="flex flex-wrap gap-2">
        {studios.map((studio: string, idx: number) => (
          <span
            key={idx}
            className="rounded-xl border border-purple-200/50 bg-gradient-to-r from-purple-50 to-pink-50 px-3 py-1.5 text-sm font-medium text-purple-700 dark:border-purple-700/30 dark:from-purple-900/20 dark:to-pink-900/20 dark:text-purple-300"
          >
            {studio}
          </span>
        ))}
      </div>
    </div>
  );
}
