export function uniqueTags(items: { tags: string[] }[]): string[] {
  const tags = new Set<string>();
  for (const item of items) {
    for (const tag of item.tags) {
      if (tag.length > 0) {
        tags.add(tag);
      }
    }
  }
  return [...tags].sort();
}

export function currentTagPrefix(query: string): string | null {
  const hash = query.lastIndexOf("#");
  if (hash < 0) {
    return null;
  }
  const after = query.slice(hash + 1);
  if (/\s/.test(after)) {
    return null;
  }
  return after;
}

export function applyTagCompletion(query: string, tag: string): string {
  const hash = query.lastIndexOf("#");
  if (hash < 0) {
    return query;
  }
  return `${query.slice(0, hash + 1)}${tag} `;
}

export function matchingTags(prefix: string, tags: string[]): string[] {
  return tags.filter((tag) => tag.startsWith(prefix)).slice(0, 8);
}

export function tagsByCount(items: { tags: string[] }[]): string[] {
  const counts = new Map<string, number>();
  for (const item of items) {
    for (const tag of item.tags) {
      if (tag.length === 0) {
        continue;
      }
      counts.set(tag, (counts.get(tag) ?? 0) + 1);
    }
  }
  return [...counts.entries()]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .map(([tag]) => tag);
}

export function isSecret(item: { tags: string[] }): boolean {
  return item.tags.includes("secret");
}

export function isLocked(item: { tags: string[] }): boolean {
  return item.tags.includes("lock");
}

export function matchesAlias(item: { tags: string[] }, text: string): boolean {
  return text.length > 0 && item.tags.includes(`alias:${text}`);
}
