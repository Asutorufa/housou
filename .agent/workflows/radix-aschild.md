---
description: Radix UI asChild composition rules with custom components
---

## Problem

When Radix UI's `asChild` is used with a custom component (e.g. `Dialog.Content`, `Dialog.Overlay`, `Select.Trigger`, etc.), failing to forward props and ref causes:

- Clicking inside the content area dismisses the Dialog (DismissableLayer broken)
- Mouse wheel scrolling inside the content is blocked (react-remove-scroll broken)
- Accessibility attributes lost (role, aria-*, etc.)

## Root Cause

Radix's `asChild` uses `Slot` to pass internal props (`ref`, `data-radix-*`, `role`, `aria-*`, event handlers, etc.) to the child component. If the child only destructures its own known props without spreading the rest onto the root DOM element, all Radix props are silently discarded.

Official docs: https://www.radix-ui.com/primitives/docs/guides/composition

## Diagnosis

**Step 1**: When any Radix component behaves unexpectedly, first check if it uses `asChild` with a custom component.
**Step 2**: Check whether that component spreads all props onto its root DOM element.
**Step 3**: Inspect the root DOM element in the browser for `data-radix-*` attributes. If missing, props are not being forwarded.

**Do NOT**: Jump to workarounds like `onInteractOutside`, `modal={false}`, `data-scroll-lock-scrollable`, or restructuring the component tree. Fix the root cause first.

## Correct Pattern

```tsx
// ❌ Wrong: only destructures own props, Radix props are lost
function MyContent({ title, onClose }: MyProps) {
  return <div>...</div>;
}

// ✅ Correct: spread remaining props onto root element (React 19: ref is included in props)
function MyContent({
  title,
  onClose,
  ...radixProps
}: MyProps & Record<string, unknown>) {
  return <div {...radixProps}>...</div>;
}
```

If you have props that should NOT be spread to the DOM (e.g. `isOpen`), destructure them explicitly:
```tsx
function MyContent({
  isOpen: _isOpen,  // destructure to prevent DOM spread
  onClose,
  ...radixProps
}: MyProps & Record<string, unknown>) {
  return <div {...radixProps}>...</div>;
}
```
