// An error boundary: a child that throws on a click is replaced by the boundary's fallback.
// proves: react.components.Component
import { Component, useState, type ReactNode } from "react";
import { trace } from "parity";
class Boundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() { return { failed: true }; }
  componentDidCatch() { trace("caught", "boundary"); }
  render() { return this.state.failed ? <div data-testid="fallback" className="h-10 w-48 bg-rose-300">failed</div> : this.props.children; }
}
function Bomb() {
  const [boom, setBoom] = useState(false);
  if (boom) throw new Error("boom");
  return <button data-testid="bomb" className="bg-slate-200 px-2" onClick={() => setBoom(true)}>throw</button>;
}
export const steps = [{ click: "bomb" }];
export default function Case() {
  return <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900"><Boundary><Bomb /></Boundary></div>;
}
