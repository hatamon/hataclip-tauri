export function parseQuery(raw: string): { tags: string[]; text: string } {
  const tags: string[] = [];
  const text = raw
    .replace(/#(\S+)/g, (_, tag: string) => {
      tags.push(tag);
      return " ";
    })
    .trim()
    .replace(/\s+/g, " ");
  return { tags, text };
}
