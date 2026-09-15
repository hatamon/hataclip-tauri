export function fuzzyScore(query: string, text: string): number | null {
  if (query.length === 0) {
    return 0;
  }
  const needle = query.toLocaleLowerCase();
  const haystack = text.toLocaleLowerCase();
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

export function fuzzyFilter<T extends { text: string }>(query: string, items: T[]): T[] {
  const scored = items
    .map((item) => ({ item, score: fuzzyScore(query, item.text) }))
    .filter((row): row is { item: T; score: number } => row.score !== null);
  scored.sort((a, b) => b.score - a.score);
  return scored.map((row) => row.item);
}
