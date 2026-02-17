import type { UniversalStaff } from "../../types";

export default function StaffSection({ staff }: { staff?: UniversalStaff[] }) {
  if (!staff || staff.length === 0) return null;

  const grouped = staff
    .slice(0, 12)
    .reduce((acc: Record<string, UniversalStaff[]>, member: UniversalStaff) => {
      const deptMap: Record<string, string> = {
        Directing: "監督・演出",
        Writing: "脚本",
        Sound: "音響",
        Camera: "撮影",
        Art: "美術",
        Production: "制作",
        "Visual Effects": "視覚効果",
        Editing: "編集",
        Lighting: "照明",
        "Costume & Make-Up": "衣装・メイク",
        Creator: "原案・原作",
        Crew: "スタッフ",
      };
      const deptEnglish = member.department || "Other";
      const dept = deptMap[deptEnglish] || deptEnglish;
      if (!acc[dept]) acc[dept] = [];
      acc[dept].push(member);
      return acc;
    }, {});

  return (
    <div className="sm:col-span-2">
      <h4 className="mb-3 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
        スタッフ
      </h4>
      {Object.entries(grouped).map(
        ([dept, members]: [string, UniversalStaff[]]) => (
          <div key={dept} className="mb-3">
            <div className="mb-1.5 text-[10px] font-bold tracking-wider text-gray-400 uppercase dark:text-gray-500">
              {dept}
            </div>
            <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
              {members.map((member: UniversalStaff, idx: number) => (
                <div
                  key={idx}
                  className="flex flex-col rounded-lg border border-gray-100 bg-gray-50 p-2 text-sm dark:border-gray-700/50 dark:bg-gray-900/40"
                >
                  <span className="truncate text-[10px] font-semibold tracking-tight text-blue-600 uppercase dark:text-blue-400">
                    {member.role}
                  </span>
                  <span className="mt-0.5 truncate font-medium text-gray-700 dark:text-gray-200">
                    {member.name}
                  </span>
                </div>
              ))}
            </div>
          </div>
        ),
      )}
    </div>
  );
}
