// localStorage and sessionStorage: setItem/getItem/removeItem/key/length/clear, values as strings, property access,
// Object.keys, and a click that counts visits kept in storage.
// proves: api.Storage, api.Storage.getItem, api.Storage.setItem, api.Storage.removeItem, api.Storage.key, api.Storage.length, api.Storage.clear, api.Window.localStorage, api.Window.sessionStorage
import { useEffect, useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "visit" }, { click: "visit" }];
export default function Case() {
  const [visits, setVisits] = useState(0);
  useEffect(() => {
    localStorage.clear();
    sessionStorage.clear();
    localStorage.setItem("a", "1");
    localStorage.setItem("n", 42 as unknown as string);
    localStorage.setItem("o", { x: 1 } as unknown as string);
    trace("local", "read", [localStorage.getItem("a"), localStorage.getItem("n"), localStorage.getItem("o"), localStorage.getItem("missing"), localStorage.length]);
    trace("local", "keys", [Object.keys(localStorage).sort(), [0, 1, 2].map((i) => localStorage.key(i)).sort(), localStorage.key(9)]);
    (localStorage as any).p = "prop";
    trace("local", "prop", [localStorage.getItem("p"), (localStorage as any).a, "a" in localStorage]);
    localStorage.removeItem("a");
    delete (localStorage as any).p;
    trace("local", "removed", [localStorage.getItem("a"), localStorage.length, localStorage instanceof Storage]);
    sessionStorage.setItem("s", "x");
    trace("session", "read", [sessionStorage.getItem("s"), sessionStorage.length, localStorage.getItem("s")]);
    localStorage.clear();
    trace("local", "cleared", localStorage.length);
  }, []);
  const visit = () => {
    const n = Number(localStorage.getItem("visits") ?? "0") + 1;
    localStorage.setItem("visits", String(n));
    setVisits(n);
  };
  return (
    <div data-testid="root" className="flex h-full w-full gap-2 bg-white p-4">
      <div data-testid="visit" className="h-8 w-16 bg-slate-300" onClick={visit} />
      <div data-testid="visits" data-value={String(visits)} className="h-8 bg-orange-400" style={{ width: 20 + visits * 30 }} />
    </div>
  );
}
