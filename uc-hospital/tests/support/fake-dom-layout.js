// Layout helpers for the browserless fake DOM.
//
// The shared timeline renderer asks for DOM geometry. These helpers keep that
// geometry deterministic without putting layout-specific details into the fake
// element implementation.

// Fallback layout width used by the shared timeline widget during tests.
function resolveInlineSize(node) {
  const styleWidth = parsePixelValue(node.style && node.style.width);
  if (styleWidth > 0) return styleWidth;
  return 1024;
}

// Fallback layout height used by the shared timeline widget during tests.
function resolveBlockSize(node) {
  const styleHeight = parsePixelValue(node.style && node.style.height);
  if (styleHeight > 0) return styleHeight;
  return 768;
}

// Child-based scroll-width approximation.
function resolveChildScrollWidth(node) {
  return node.children.reduce((maxWidth, child) => Math.max(maxWidth, child.scrollWidth), 0);
}

// Child-based scroll-height approximation.
function resolveChildScrollHeight(node) {
  return node.children.reduce((maxHeight, child) => Math.max(maxHeight, child.scrollHeight), 0);
}

// Parses inline `px` sizes from the fake style object.
function parsePixelValue(value) {
  if (typeof value === 'number') return value;
  const match = String(value || '').match(/^(-?\d+(?:\.\d+)?)px$/);
  return match ? Number(match[1]) : 0;
}

module.exports = {
  resolveInlineSize,
  resolveBlockSize,
  resolveChildScrollWidth,
  resolveChildScrollHeight,
};
