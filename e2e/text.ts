export function plain(text: string): string {
  return text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

export const DATE = /^\d{4}\/\d{2}\/\d{2}$/;
export const TIME = /^\d{2}:\d{2}$/;
export const UUID =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
