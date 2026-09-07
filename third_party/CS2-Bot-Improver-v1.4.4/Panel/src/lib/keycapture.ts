/** Map a browser KeyboardEvent to a Source-engine bind key name. */
export function captureKeyName(e: KeyboardEvent): string {
  const code = e.code;

  if (/^Key[A-Z]$/.test(code)) return code.slice(3).toLowerCase();
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^Numpad[0-9]$/.test(code)) return "kp_" + code.slice(6);
  if (/^F([1-9]|1[0-2])$/.test(code)) return code.toLowerCase();

  const map: Record<string, string> = {
    Backslash: "\\",
    Space: "space",
    Enter: "enter",
    Tab: "tab",
    Escape: "escape",
    Backspace: "backspace",
    Semicolon: "semicolon",
    Quote: "'",
    Comma: ",",
    Period: ".",
    Slash: "/",
    Minus: "-",
    Equal: "=",
    BracketLeft: "[",
    BracketRight: "]",
    Backquote: "`",
    CapsLock: "capslock",
    ArrowUp: "uparrow",
    ArrowDown: "downarrow",
    ArrowLeft: "leftarrow",
    ArrowRight: "rightarrow",
    Insert: "ins",
    Delete: "del",
    Home: "home",
    End: "end",
    PageUp: "pgup",
    PageDown: "pgdn",
    ShiftLeft: "shift",
    ShiftRight: "rshift",
    ControlLeft: "ctrl",
    ControlRight: "rctrl",
    AltLeft: "alt",
    AltRight: "ralt",
  };

  return map[code] ?? e.key.toLowerCase();
}
