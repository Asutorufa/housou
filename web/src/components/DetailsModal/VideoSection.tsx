import type { UniversalVideo } from "../../types";

export default function VideoSection({
  videos,
}: {
  videos?: UniversalVideo[];
}) {
  if (!videos || videos.length === 0) return null;

  // Filter for YouTube videos as they are the most common and easy to embed/link
  const youtubeVideos = videos.filter((v) => v.site === "YouTube" && v.key);

  if (youtubeVideos.length === 0) return null;

  return (
    <div className="mb-6">
      <h4 className="mb-3 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
        映像特典・PV
      </h4>
      <div className="flex gap-4 overflow-x-auto pb-2 custom-scrollbar snap-x snap-mandatory">
        {youtubeVideos.map((video) => (
          <a
            key={video.key}
            href={`https://www.youtube.com/watch?v=${video.key}`}
            target="_blank"
            rel="noopener noreferrer"
            className="snap-start flex-none w-64 group relative overflow-hidden rounded-xl bg-black aspect-video ring-1 ring-white/10 hover:ring-2 hover:ring-blue-500 transition-all focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <img
              src={`https://img.youtube.com/vi/${video.key}/mqdefault.jpg`}
              alt={video.name || "Video thumbnail"}
              className="h-full w-full object-cover opacity-80 transition-opacity group-hover:opacity-100"
              loading="lazy"
            />
            <div className="absolute inset-0 flex items-center justify-center transition-transform duration-300 group-hover:scale-110">
              <div className="rounded-full bg-white/20 p-3 backdrop-blur-sm group-hover:bg-white/30">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                  className="h-8 w-8 text-white drop-shadow-md"
                >
                  <path
                    fillRule="evenodd"
                    d="M4.5 5.653c0-1.426 1.529-2.33 2.779-1.643l11.54 6.348c1.295.712 1.295 2.573 0 3.285L7.28 19.991c-1.25.687-2.779-.217-2.779-1.643V5.653z"
                    clipRule="evenodd"
                  />
                </svg>
              </div>
            </div>
            <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/90 via-black/60 to-transparent p-3 pt-8">
              {video.type && (
                <span className="mb-1 inline-block rounded bg-blue-600/80 px-1.5 py-0.5 text-[10px] font-bold text-white backdrop-blur-sm">
                  {video.type}
                </span>
              )}
              {video.name && (
                <p className="text-xs font-bold text-white line-clamp-2 drop-shadow-md">
                  {video.name}
                </p>
              )}
            </div>
          </a>
        ))}
      </div>
    </div>
  );
}
