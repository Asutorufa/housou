import type { UniversalCharacter } from "../../types";

export default function CastSection({
  characters,
}: {
  characters?: UniversalCharacter[];
}) {
  if (!characters || characters.length === 0) return null;

  return (
    <div className="sm:col-span-2">
      <h4 className="mb-3 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
        キャスト
      </h4>
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {characters.slice(0, 6).map((char, idx: number) => (
          <div
            key={idx}
            className="group flex flex-col rounded-2xl border border-gray-100 bg-gray-50 p-3 transition-colors hover:border-blue-200 dark:border-gray-700/50 dark:bg-gray-900/40 dark:hover:border-blue-900/50"
          >
            <div className="truncate font-bold text-gray-800 transition-colors group-hover:text-blue-600 dark:text-gray-100 dark:group-hover:text-blue-400">
              {char.name}
            </div>
            {char.voiceActor && (
              <div className="mt-1 flex items-center gap-1.5">
                <span className="rounded bg-gray-200 px-1 py-0.5 text-[9px] font-black tracking-tighter text-gray-500 uppercase dark:bg-gray-800 dark:text-gray-400">
                  CV
                </span>
                <span className="truncate text-xs text-gray-500 dark:text-gray-400">
                  {char.voiceActor}
                </span>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
