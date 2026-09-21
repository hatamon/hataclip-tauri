const PICK = /\{\{\s*pick:([^}]*)\}\}/g;

export type PickSpec = { spec: string; options: string[] };

export function pickSpecs(text: string): PickSpec[] {
  const specs: PickSpec[] = [];
  PICK.lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = PICK.exec(text)) !== null) {
    const spec = match[1].trim();
    if (specs.some((entry) => entry.spec === spec)) {
      continue;
    }
    const options = spec
      .split(",")
      .map((part) => part.trim())
      .filter((part) => part.length > 0);
    specs.push({ spec, options });
  }
  return specs;
}

export function uniquePickSpecs(items: { text: string }[]): PickSpec[] {
  const specs: PickSpec[] = [];
  for (const item of items) {
    for (const entry of pickSpecs(item.text)) {
      if (!specs.some((existing) => existing.spec === entry.spec)) {
        specs.push(entry);
      }
    }
  }
  return specs;
}
