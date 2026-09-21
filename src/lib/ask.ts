const ASK = /\{\{\s*ask:([^}]*)\}\}/g;

export function askNames(text: string): string[] {
  const names: string[] = [];
  ASK.lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = ASK.exec(text)) !== null) {
    const name = match[1].trim();
    if (!names.includes(name)) {
      names.push(name);
    }
  }
  return names;
}

export function uniqueAskNames(items: { text: string }[]): string[] {
  const names: string[] = [];
  for (const item of items) {
    for (const name of askNames(item.text)) {
      if (!names.includes(name)) {
        names.push(name);
      }
    }
  }
  return names;
}
