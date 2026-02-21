export class ApiError extends Error {
  status?: number;

  constructor(message: string, status?: number) {
    super(message);
    this.status = status;
    this.name = "ApiError";
  }
}

export const fetcher = async (url: string) => {
  const res = await fetch(url);
  if (res.status === 401) {
    const error = new ApiError("Unauthorized", 401);
    throw error;
  }
  if (!res.ok) {
    let errorInfo = "";
    try {
      errorInfo = await res.text();
    } catch {
      // Ignore
    }
    throw new Error(
      `Failed to fetch: ${res.status} ${res.statusText}${
        errorInfo ? ` - ${errorInfo}` : ""
      }`,
    );
  }
  return res.json();
};
