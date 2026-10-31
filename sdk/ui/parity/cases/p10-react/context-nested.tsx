// Nested providers: the nearest provider wins; a consumer outside the inner one sees the outer value.
// proves: react.hooks.useContext, react.api.createContext
import { createContext, useContext } from "react";
const Size = createContext(10);
function Bar({ id }: { id: string }) {
  const s = useContext(Size);
  return <div data-testid={id} data-value={String(s)} className="mb-1 h-4 bg-teal-500" style={{ width: s * 10 }} />;
}
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <Bar id="default" />
      <Size.Provider value={20}>
        <Bar id="outer" />
        <Size.Provider value={30}><Bar id="inner" /></Size.Provider>
      </Size.Provider>
    </div>
  );
}
