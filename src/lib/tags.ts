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
