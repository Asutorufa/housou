export class ApiError extends Error {
  status?: number;

  constructor(message: string, status?: number) {
    super(message);
    this.status = status;
    this.name = "ApiError";
  }
}

export async function checkResponse(res: Response): Promise<Response> {
  if (!res.ok) {
    let message =
      res.status === 401 ? "Unauthorized" : `Request failed (${res.status})`;
    try {
      const body = await res.text();
      if (body) {
        try {
          const json: { error?: string; message?: string } = JSON.parse(body);
          message = json.error || json.message || message;
        } catch {
          message = body;
        }
      }
    } catch {
      // Preserve the HTTP error if its body cannot be read.
    }
    throw new ApiError(message, res.status);
  }
  return res;
}

export async function fetcher<T = unknown>(
  url: string,
  init?: RequestInit,
): Promise<T> {
  const res = await checkResponse(await fetch(url, init));
  return res.json() as Promise<T>;
}
