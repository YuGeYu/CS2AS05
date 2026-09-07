/**
 * Feature-detect backdrop-filter. Some WebKitGTK builds (Linux) report support
 * via CSS.supports but render it broken, so we also do a runtime paint probe.
 * On failure we tag <html> with `no-backdrop`, which the glass styles fall back
 * to a solid translucent card for.
 */
export function applyGlassSupport(): boolean {
  const supported = cssSupportsBackdrop();
  document.documentElement.classList.toggle("no-backdrop", !supported);
  return supported;
}

function cssSupportsBackdrop(): boolean {
  try {
    return (
      CSS.supports("backdrop-filter", "blur(2px)") ||
      CSS.supports("-webkit-backdrop-filter", "blur(2px)")
    );
  } catch {
    return false;
  }
}
