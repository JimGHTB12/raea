// The same panel through React (the adapter): one useState per piece, rows as plain objects. With `memo` the rows are
// React.memo with stable callbacks (the careful version); without it, as most React is written.
import { memo, useCallback, useState } from "react";

type Row = { id: string; label: string; done: boolean };
const ROW = "flex flex-row gap-2 p-1 rounded bg-ink/10";
const ROW_DONE = "flex flex-row gap-2 p-1 rounded bg-ink/5";

function RowView({ r, onToggle, onRemove }: { r: Row; onToggle: (id: string) => void; onRemove: (id: string) => void }) {
  return (
    <node data-testid={r.id} className={r.done ? ROW_DONE : ROW} aria-selected={r.done} onPointerDown={() => onToggle(r.id)}>
      <text>{r.label}</text>
      <node data-testid={`x-${r.id}`} onPointerDown={() => onRemove(r.id)}>
        <text>x</text>
      </node>
    </node>
  );
}
const RowMemo = memo(RowView);

export function Tasks({ n, useMemo }: { n: number; useMemo: boolean }) {
  const [next] = useState(() => ({ i: 0 }));
  const mk = (label?: string): Row => {
    const i = next.i++;
    return { id: `r${i}`, label: label ?? `Task ${i}`, done: false };
  };
  const [rows, setRows] = useState<Row[]>(() => Array.from({ length: n }, () => mk()));
  const [draft, setDraft] = useState("");
  const [compact, setCompact] = useState(false);
  const left = rows.filter((r) => !r.done).length;
  const R = useMemo ? RowMemo : RowView;

  const onToggle = useCallback((id: string) => setRows((rs) => rs.map((r) => (r.id === id ? { ...r, done: !r.done } : r))), []);
  const onRemove = useCallback((id: string) => setRows((rs) => rs.filter((r) => r.id !== id)), []);
  const add = () => {
    const t = draft.trim();
    if (!t) return;
    setRows((rs) => [...rs, mk(t)]);
    setDraft("");
  };

  return (
    <node data-testid="panel" className="flex flex-col gap-2 p-2">
      <node className="flex flex-row gap-2 items-center">
        <text className="text-[14px]">Tasks</text>
        <text data-testid="left">{`${left} left`}</text>
        <node data-testid="toggle" role="button" aria-pressed={compact} className="px-2 rounded bg-ink/10" onPointerDown={() => setCompact((c) => !c)}>
          <text>{compact ? "Wide" : "Compact"}</text>
        </node>
        <node data-testid="rotate" onPointerDown={() => setRows((rs) => [rs[rs.length - 1], ...rs.slice(0, -1)])}>
          <text>rotate</text>
        </node>
        <node data-testid="unrotate" onPointerDown={() => setRows((rs) => [...rs.slice(1), rs[0]])}>
          <text>unrotate</text>
        </node>
        <node
          data-testid="all"
          onPointerDown={() =>
            setRows((rs) => {
              const v = rs.some((r) => !r.done);
              return rs.map((r) => (r.done === v ? r : { ...r, done: v }));
            })
          }
        >
          <text>all</text>
        </node>
      </node>
      <editableText data-testid="draft" value={draft} onChange={(t: string) => setDraft(t)} onKeyDown={(e: { key: string }) => e.key === "Enter" && add()} />
      <node data-testid="list" className={compact ? "flex flex-row flex-wrap gap-1" : "flex flex-col gap-2"}>
        {rows.map((r) => (
          <R key={r.id} r={r} onToggle={onToggle} onRemove={onRemove} />
        ))}
      </node>
      {left === 0 && <text data-testid="alldone">All done</text>}
    </node>
  );
}
