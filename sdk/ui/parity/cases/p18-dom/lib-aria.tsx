// P18, a real library: React Aria's hooks (react-aria 3.53.1, react-stately) for a button, a listbox and a combobox,
// driven by presses, keys and typing. usePress, useFocusRing, FocusScope, useSelectableList, useComboBox, useFilter
// and useOverlay run on the DOM-like surface: pointer and key events through capture and bubble, focus, the
// TreeWalker, contains, getComputedStyle and scrollIntoView.
// proves: api.PointerEvent.pointerType, api.Event.currentTarget, api.Node.contains, api.Document.activeElement, api.Element.scrollIntoView
import { useRef } from "react";
import { useButton, useComboBox, useFilter, useListBox, useOption } from "react-aria";
import { Item, useComboBoxState, useListState } from "react-stately";
import { trace } from "parity";

export const steps = [
  { click: "btn" }, { key: "Enter" },
  { click: "opt-b" }, { key: "ArrowDown" }, { key: " " },
  { click: "combo" }, { text: "an" }, { wait: 4 }, { key: "ArrowDown" }, { key: "Enter" }, { wait: 4 },
];

// options draw no text (text is P13's); the collection reads each item's textValue
const tid = () => document.activeElement?.getAttribute("data-testid") ?? "body";

function Button() {
  const ref = useRef<HTMLButtonElement>(null);
  const { buttonProps } = useButton({ onPress: (e) => trace("press", "btn", [e.pointerType, tid()]), onPressStart: (e) => trace("pressStart", "btn", e.pointerType) }, ref);
  return <button {...buttonProps} ref={ref} data-testid="btn" style={{ width: 90, height: 28, backgroundColor: "rgb(30, 41, 59)", flexShrink: 0, outline: "none" }} />;
}

function Option({ item, state }: { item: any; state: any }) {
  const ref = useRef<HTMLLIElement>(null);
  const { optionProps, isSelected, isFocused } = useOption({ key: item.key }, state, ref);
  return <li {...optionProps} ref={ref} data-testid={`opt-${item.key}`} style={{ height: 24, outline: "none", backgroundColor: isSelected ? "rgb(14, 165, 233)" : isFocused ? "rgb(203, 213, 225)" : "rgb(241, 245, 249)" }} />;
}

function ListBox() {
  const state = useListState({ selectionMode: "single", onSelectionChange: (k) => trace("selected", "list", [...k]), children: ["a", "b", "c", "d"].map((k) => <Item key={k} textValue={k}>{k}</Item>) } as any);
  const ref = useRef<HTMLUListElement>(null);
  const { listBoxProps } = useListBox({ "aria-label": "letters", selectionMode: "single" } as any, state, ref);
  return (
    <ul {...listBoxProps} ref={ref} data-testid="list" className="flex flex-col" style={{ width: 120, flexShrink: 0, outline: "none" }}>
      {[...state.collection].map((item) => (
        <Option key={item.key} item={item} state={state} />
      ))}
    </ul>
  );
}

// the popover is a flex column: the host lays a plain div out as a flex row where Chrome has a block (P12)
const FRUIT = ["apple", "banana", "cherry", "mango", "orange"];
function ComboBox() {
  const { contains } = useFilter({ sensitivity: "base" });
  const state = useComboBoxState({ defaultFilter: contains, onSelectionChange: (k) => trace("combo-selected", "combo", k), children: FRUIT.map((f) => <Item key={f} textValue={f}>{f}</Item>) } as any);
  const inputRef = useRef<HTMLInputElement>(null);
  const listBoxRef = useRef<HTMLUListElement>(null);
  const popoverRef = useRef<HTMLDivElement>(null);
  const { inputProps, listBoxProps } = useComboBox({ "aria-label": "fruit", inputRef, listBoxRef, popoverRef } as any, state);
  return (
    <div className="flex flex-col" style={{ flexShrink: 0 }}>
      <input {...inputProps} ref={inputRef} data-testid="combo" style={{ width: 160, height: 28, backgroundColor: "rgb(241, 245, 249)", color: "rgb(241, 245, 249)", caretColor: "transparent", outline: "none" }} />
      {state.isOpen && (
        <div ref={popoverRef} data-testid="combo-pop" className="flex flex-col">
          <ComboList listBoxProps={listBoxProps} listBoxRef={listBoxRef} state={state} />
        </div>
      )}
    </div>
  );
}
function ComboList({ listBoxProps, listBoxRef, state }: any) {
  const { listBoxProps: lp } = useListBox(listBoxProps, state, listBoxRef);
  trace("combo-items", "combo", [...state.collection].map((i: any) => i.key));
  return (
    <ul {...lp} ref={listBoxRef} className="flex flex-col">
      {[...state.collection].map((item: any) => (
        <Option key={item.key} item={item} state={state} />
      ))}
    </ul>
  );
}

export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full flex-col bg-white" style={{ padding: 16, gap: 8 }}>
      <Button />
      <ListBox />
      <ComboBox />
    </div>
  );
}
