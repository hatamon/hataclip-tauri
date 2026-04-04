/** Payload from Rust `clipboard-history-updated` event. */
export interface ClipboardHistoryPayload {
  items: string[];
}

export function isClipboardHistoryPayload(
  value: unknown
): value is ClipboardHistoryPayload {
  return (
    typeof value === "object" &&
    value !== null &&
    "items" in value &&
    Array.isArray((value as ClipboardHistoryPayload).items) &&
    (value as ClipboardHistoryPayload).items.every((i) => typeof i === "string")
  );
}
