// History and in-memory routing: pushState and replaceState with state, popstate on back and forward (after a task,
// not during the call), hashchange, and a small router that renders the page the URL names. The page's own path
// differs between the sides (a file in Chrome), so only the search, the hash, the state and the length are compared.
// proves: api.History, api.History.pushState, api.History.replaceState, api.History.back, api.History.forward, api.History.go, api.History.length, api.History.state, api.Location, api.Location.search, api.Location.hash, api.Window.history, api.Window.location
import { useEffect, useState } from "react";
import { trace } from "parity";
const page = () => new URLSearchParams(location.search).get("page") ?? "home";
export const steps = [{ click: "to-a" }, { click: "to-b" }, { click: "back" }, { wait: 4 }, { click: "back" }, { wait: 4 }, { click: "forward" }, { wait: 4 }, { click: "hash" }, { wait: 4 }, { click: "replace" }];
export default function Case() {
  const [p, setP] = useState(page());
  useEffect(() => {
    const base = history.length;
    const pop = (e: PopStateEvent) => {
      trace("popstate", page(), [e.state, location.search, location.hash, history.length - base]);
      setP(page());
    };
    const hash = () => trace("hashchange", "window", location.hash);
    window.addEventListener("popstate", pop);
    window.addEventListener("hashchange", hash);
    return () => {
      window.removeEventListener("popstate", pop);
      window.removeEventListener("hashchange", hash);
    };
  }, []);
  const push = (to: string) => {
    history.pushState({ to }, "", "?page=" + to);
    trace("push", to, [history.state, location.search]);
    setP(page());
  };
  const tabs: [string, () => void, string][] = [
    ["to-a", () => push("a"), "bg-sky-300"],
    ["to-b", () => push("b"), "bg-sky-400"],
    ["back", () => { history.back(); trace("back", "sync", location.search); }, "bg-slate-300"],
    ["forward", () => history.forward(), "bg-slate-400"],
    ["hash", () => { location.hash = "#sec"; trace("hash", "sync", location.hash); }, "bg-amber-300"],
    ["replace", () => { history.replaceState({ r: 1 }, "", "?page=r"); trace("replace", "r", [history.state, location.search, location.hash]); setP(page()); }, "bg-rose-300"],
  ];
  return (
    <div data-testid="root" className="flex h-full w-full flex-col gap-2 bg-white p-4">
      <div className="flex gap-2">{tabs.map(([id, fn, c]) => <div key={id} data-testid={id} className={"h-8 w-12 " + c} onClick={fn} />)}</div>
      <div data-testid="page" data-value={p} className={{ home: "h-16 w-32 bg-slate-200", a: "h-16 w-48 bg-indigo-300", b: "h-16 w-64 bg-emerald-300", r: "h-16 w-80 bg-rose-300" }[p] ?? "h-16 w-16 bg-black"} />
    </div>
  );
}
