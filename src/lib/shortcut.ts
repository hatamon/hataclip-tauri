export type Shortcuts = {
  register: string;
  show: string;
};

type KeyCombo = Pick<KeyboardEvent, "code" | "ctrlKey" | "altKey" | "shiftKey" | "metaKey">;

const MODIFIER_CODES = /^(Control|Shift|Alt|Meta|OS)(Left|Right)?$/;

const MODIFIER_LABELS: Record<string, string> = {
  Control: "Ctrl",
  Alt: "Alt",
  Shift: "Shift",
  Super: "Win",
};

export function fromEvent(event: KeyCombo): string | null {
  if (MODIFIER_CODES.test(event.code)) {
    return null;
  }
  const modifiers: string[] = [];
  if (event.ctrlKey) {
    modifiers.push("Control");
  }
  if (event.altKey) {
    modifiers.push("Alt");
  }
  if (event.shiftKey) {
    modifiers.push("Shift");
  }
  if (event.metaKey) {
    modifiers.push("Super");
  }
  if (modifiers.length === 0) {
    return null;
  }
  return [...modifiers, event.code].join("+");
}

export function toLabel(value: string): string {
  const tokens = value.split("+");
  const key = tokens.pop();
  if (key === undefined) {
    return value;
  }
  return [...tokens.map((token) => MODIFIER_LABELS[token] ?? token), keyLabel(key)].join(" + ");
}

function keyLabel(code: string): string {
  const digit = /^Digit(\d)$/.exec(code);
  if (digit) {
    return digit[1];
  }
  const letter = /^Key([A-Z])$/.exec(code);
  if (letter) {
    return letter[1];
  }
  const numpad = /^Numpad(\d)$/.exec(code);
  if (numpad) {
    return `Num${numpad[1]}`;
  }
  return code;
}
