// P18, a real library: Radix's Dialog, Popover and DropdownMenu (radix-ui 1.8.1) mounted as an app would, driven by
// presses and keys. Its FocusScope, DismissableLayer, Portal, Presence, RovingFocusGroup, Collection and Popper
// (floating-ui) run on the DOM-like surface: document listeners, TreeWalker focus, createPortal into body,
// getComputedStyle, getBoundingClientRect, ResizeObserver.
// proves: api.Document.activeElement, api.HTMLElement.focus, api.EventTarget.addEventListener, api.Document.createTreeWalker, api.Window.getComputedStyle, api.ResizeObserver
import { useState } from "react";
import { Dialog, DropdownMenu, Popover } from "radix-ui";
import { trace } from "parity";

export const steps = [
  { click: "dlg-open" }, { wait: 4 }, { key: "Escape" }, { wait: 4 },
  { click: "pop-open" }, { wait: 4 }, { click: "outside" }, { wait: 4 },
  { click: "menu-open" }, { wait: 4 }, { key: "ArrowDown" }, { wait: 2 }, { key: "ArrowDown" }, { wait: 2 }, { key: "Enter" }, { wait: 4 },
];

// where the focus is after each step is the harness's own check (each element's focus), not a trace: Radix moves it
// in a timer, and when a rAF reads it is timing, not behaviour. No focus ring (outline is P12's).
const box = (bg: string, w = 90) => ({ width: w, height: 28, backgroundColor: bg, flexShrink: 0, outline: "none" });

export default function Case() {
  const [dlg, setDlg] = useState(false);
  const [pop, setPop] = useState(false);
  const [menu, setMenu] = useState(false);
  return (
    <div data-testid="root" className="flex h-full w-full flex-col bg-white" style={{ padding: 16, gap: 8 }}>
      <Dialog.Root open={dlg} onOpenChange={(o) => (trace("dialog", "open", o), setDlg(o))}>
        <Dialog.Trigger data-testid="dlg-open" style={box("rgb(30, 41, 59)")} />
        <Dialog.Portal>
          <Dialog.Overlay data-testid="dlg-overlay" />
          <Dialog.Content data-testid="dlg" aria-describedby={undefined} style={{ width: 200, height: 100, backgroundColor: "rgb(241, 245, 249)" }}>
            <Dialog.Title style={{ height: 0, overflow: "hidden" }} />
            <button data-testid="dlg-first" style={box("rgb(225, 29, 72)")} />
            <Dialog.Close data-testid="dlg-close" style={box("rgb(14, 165, 233)")} />
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
      <Popover.Root open={pop} onOpenChange={(o) => (trace("popover", "open", o), setPop(o))}>
        <Popover.Trigger data-testid="pop-open" style={box("rgb(124, 58, 237)")} />
        <Popover.Portal>
          <Popover.Content data-testid="pop" side="bottom" align="start" style={{ width: 160, height: 60, backgroundColor: "rgb(16, 185, 129)" }}>
            <button data-testid="pop-btn" style={box("rgb(234, 88, 12)", 60)} />
          </Popover.Content>
        </Popover.Portal>
      </Popover.Root>
      <DropdownMenu.Root open={menu} onOpenChange={(o) => (trace("menu", "open", o), setMenu(o))}>
        <DropdownMenu.Trigger data-testid="menu-open" style={box("rgb(202, 138, 4)")} />
        <DropdownMenu.Portal>
          <DropdownMenu.Content data-testid="menu" side="bottom" align="start" style={{ width: 140, backgroundColor: "rgb(241, 245, 249)" }}>
            {["one", "two", "three"].map((k) => (
              <DropdownMenu.Item key={k} data-testid={`item-${k}`} onSelect={() => trace("select", k)} textValue={k} style={{ height: 24, outline: "none" }} />
            ))}
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>
      <div style={{ height: 120, flexShrink: 0 }} />
      <div data-testid="outside" style={{ width: 300, height: 60, backgroundColor: "rgb(203, 213, 225)", flexShrink: 0 }} />
    </div>
  );
}
