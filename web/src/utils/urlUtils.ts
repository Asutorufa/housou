/**
 * Validates if a string is a valid URL with http or https protocol.
 * This is used to sanitize external links and prevent XSS (e.g. javascript: URLs).
 *
 * @param urlString The URL string to validate
 * @returns true if the URL is valid and uses http or https protocol, false otherwise
 */
export const isValidUrl = (urlString: string | undefined | null): boolean => {
  if (!urlString) return false;

  try {
    const url = new URL(urlString);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
};
