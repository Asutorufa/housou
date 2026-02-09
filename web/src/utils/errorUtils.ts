/**
 * Safely extracts an error message from an unknown error object.
 */
export const getErrorMessage = (error: unknown): string => {
  if (error instanceof Error) return error.message;
  return String(error);
};
