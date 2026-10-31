// The shelf panel through React (the adapter): the same tree, React's way.
import { createContext, useContext, useRef, useState } from "react";
import type { Instance } from "../../src/react";

type Item = { id: string; name: string };
const Theme = createContext<"dark" | "light">("dark");

function ItemRow({ it, locked, pick }: { it: Item; locked: boolean; pick: (id: string) => void }) {
  const theme = useContext(Theme);
  return (
    <node data-testid={it.id} className={theme === "dark" ? "p-1 bg-ink/10" : "p-1 bg-white"} onPointerDown={locked ? undefined : () => pick(it.id)}>
      <text>{it.name}</text>
    </node>
  );
}

export function Shelf() {
  const [theme, setTheme] = useState<"dark" | "light">("dark");
  const [grid, setGrid] = useState(false);
  const [width, setWidth] = useState(100);
  const [padded, setPadded] = useState(false);
  const [who, setWho] = useState("you");
  const [locked, setLocked] = useState(false);
  const [picked, setPicked] = useState("");
  const [items, setItems] = useState<Item[]>(() => [0, 1, 2, 3, 4].map((i) => ({ id: `i${i}`, name: `Item ${i}` })));
  const [draft, setDraft] = useState("");
  const field = useRef<Instance>(null);
  const made = useRef(5);
  return (
    <Theme.Provider value={theme}>
      <node data-testid="shelf" className={theme === "dark" ? "bg-bg text-ink" : "bg-white text-bg"}>
        <node data-testid="theme" onPointerDown={() => setTheme((t) => (t === "dark" ? "light" : "dark"))}>
          <text>{theme}</text>
        </node>
        <node data-testid="mode" onPointerDown={() => setGrid((g) => !g)} />
        <node data-testid="grow" style={{ width, height: 20 }} onPointerDown={() => setWidth((w) => w + 10)} />
        <node data-testid="pad" style={padded ? { padding: 4 } : undefined} onPointerDown={() => setPadded((p) => !p)} />
        <text data-testid="rich">
          Hello <text className="font-bold">{who}</text>!
        </text>
        <node data-testid="rename" onPointerDown={() => setWho((w) => (w === "you" ? "Shane" : "you"))} />
        <node data-testid="lock" aria-pressed={locked} onPointerDown={() => setLocked((l) => !l)} />
        <node
          data-testid="front"
          onPointerDown={() => {
            const k = made.current++;
            setItems((xs) => [{ id: `i${k}`, name: `Item ${k}` }, ...xs]);
          }}
        />
        <node data-testid="cut" onPointerDown={() => setItems((xs) => xs.filter((_, k) => k !== 2))} />
        <node data-testid="focus" onPointerDown={() => field.current?.focus()} />
        <editableText data-testid="field" ref={field} value={draft} onChange={(t: string) => setDraft(t)} />
        <text data-testid="picked">{picked || "none"}</text>
        {padded ? (
          <node data-testid="more">
            <text>More</text>
          </node>
        ) : (
          <text data-testid="less">Less</text>
        )}
        {grid ? <node data-testid="grid" className="flex-row flex-wrap" /> : <text data-testid="list">a list</text>}
        <node data-testid="items">
          {items.map((it) => (
            <ItemRow key={it.id} it={it} locked={locked} pick={setPicked} />
          ))}
        </node>
      </node>
    </Theme.Provider>
  );
}
