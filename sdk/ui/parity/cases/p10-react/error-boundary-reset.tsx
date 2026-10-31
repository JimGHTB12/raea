// An error boundary's whole life: getDerivedStateFromError and componentDidCatch (with a component stack), a nested
// boundary catching before the outer one, an error in an effect caught as one in render, and a reset that remounts.
// proves: react.components.Component
import { Component, useEffect, useState, type ReactNode } from "react";
import { trace } from "parity";
class Boundary extends Component<{ name: string; children: ReactNode }, { error: string | null }> {
  state = { error: null as string | null };
  static getDerivedStateFromError(e: Error) { return { error: e.message }; }
  componentDidCatch(e: Error, info: { componentStack?: string }) { trace("caught", this.props.name, { message: e.message, stack: typeof info.componentStack === "string" }); }
  render() {
    if (this.state.error)
      return <button data-testid={"reset-" + this.props.name} data-value={this.state.error} className="h-8 w-48 bg-rose-300" onClick={() => { renderBomb = effectBomb = false; this.setState({ error: null }); }}>{this.state.error}</button>;
    return this.props.children;
  }
}
let renderBomb = false;
let effectBomb = false;
function Bomb() {
  if (renderBomb) throw new Error("in render");
  return <div data-testid="ok" className="h-8 w-32 bg-emerald-300" />;
}
function Mine() {
  useEffect(() => {
    if (effectBomb) throw new Error("in effect");
  });
  return <div data-testid="mine" className="h-8 w-32 bg-amber-200" />;
}
export const steps = [{ click: "render" }, { click: "reset-inner" }, { click: "effect" }, { click: "reset-outer" }];
export default function Case() {
  const [, force] = useState(0);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div className="flex gap-2">
        <button data-testid="render" className="bg-slate-200 px-2" onClick={() => { renderBomb = true; force((n) => n + 1); }}>render</button>
        <button data-testid="effect" className="bg-slate-200 px-2" onClick={() => { renderBomb = false; effectBomb = true; force((n) => n + 1); }}>effect</button>
      </div>
      <Boundary name="outer">
        <Mine />
        <Boundary name="inner"><Bomb /></Boundary>
      </Boundary>
    </div>
  );
}
