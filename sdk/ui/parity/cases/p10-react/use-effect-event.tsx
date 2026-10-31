// useEffectEvent (React 19.2): an effect's event sees the latest props without the effect depending on them.
// proves: react.hooks.useEffectEvent
import { useEffect, useEffectEvent, useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "theme" }, { click: "room" }, { click: "theme" }];
export default function Case() {
  const [room, setRoom] = useState(1);
  const [theme, setTheme] = useState("light");
  const onConnect = useEffectEvent((r: number) => trace("connected", "room", { room: r, theme }));
  useEffect(() => {
    onConnect(room);
    return () => trace("disconnected", "room", room);
  }, [room]);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div className="flex gap-2">
        <button data-testid="theme" className="bg-slate-200 px-2" onClick={() => setTheme(theme === "light" ? "dark" : "light")}>theme</button>
        <button data-testid="room" className="bg-slate-200 px-2" onClick={() => setRoom(room + 1)}>room</button>
      </div>
      <div data-testid="state" data-value={`${room} ${theme}`} className={theme === "dark" ? "mt-2 h-8 w-40 bg-slate-700" : "mt-2 h-8 w-40 bg-slate-200"} />
    </div>
  );
}
