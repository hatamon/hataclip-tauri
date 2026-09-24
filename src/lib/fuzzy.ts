const SEARCH_BYTES = 4096;

export function searchHay(text: string): string {
  const bytes = new TextEncoder().encode(text);
  if (bytes.length <= SEARCH_BYTES) {
    return text.toLowerCase();
  }
  let end = SEARCH_BYTES;
  while (end > 0 && (bytes[end] & 0xc0) === 0x80) {
    end -= 1;
  }
  return new TextDecoder().decode(bytes.subarray(0, end)).toLowerCase();
}

export function fuzzyScore(query: string, text: string, hay = searchHay(text)): number | null {
  if (query.length === 0) {
    return 0;
  }
  const needle = query.toLowerCase();
  const haystack = hay;
  let qi = 0;
  let score = 0;
  let consecutive = 0;
  for (let i = 0; i < haystack.length && qi < needle.length; i += 1) {
    if (haystack[i] === needle[qi]) {
      consecutive += 1;
      score += 1 + consecutive;
      qi += 1;
    } else {
      consecutive = 0;
    }
  }
  return qi === needle.length ? score : null;
}

export function fuzzyFilter<T extends { text: string }>(
  query: string,
  items: T[],
  hayOf?: (item: T) => string,
): T[] {
  const scored = items
    .map((item) => ({ item, score: fuzzyScore(query, item.text, hayOf?.(item)) }))
    .filter((row): row is { item: T; score: number } => row.score !== null);
  scored.sort((a, b) => b.score - a.score);
  return scored.map((row) => row.item);
}
