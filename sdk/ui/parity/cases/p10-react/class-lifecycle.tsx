// A class component's lifecycle in order: getDerivedStateFromProps, shouldComponentUpdate, getSnapshotBeforeUpdate,
// componentDidMount/DidUpdate/WillUnmount, setState's callback, forceUpdate, and a PureComponent skipping equal props.
// proves: react.components.Component, react.components.PureComponent
import { Component, PureComponent, useState } from "react";
import { trace } from "parity";
class Life extends Component<{ n: number }, { seen: number; local: number }> {
  state = { seen: -1, local: 0 };
  static getDerivedStateFromProps(p: { n: number }, s: { seen: number }) {
    trace("derive", "life", { n: p.n, seen: s.seen });
    return p.n === s.seen ? null : { seen: p.n };
  }
  shouldComponentUpdate(p: { n: number }, s: { local: number }) {
    const ok = p.n !== 3;
    trace("should", "life", { n: p.n, local: s.local, ok });
    return ok;
  }
  getSnapshotBeforeUpdate() {
    trace("snapshot", "life");
    return this.state.local;
  }
  componentDidMount() { trace("didMount", "life"); }
  componentDidUpdate(_: unknown, prev: { local: number }, snap: number) { trace("didUpdate", "life", { prev: prev.local, snap }); }
  componentWillUnmount() { trace("willUnmount", "life"); }
  render() {
    trace("render", "life", { n: this.props.n, local: this.state.local });
    return (
      <div data-testid="life" data-value={`${this.props.n}/${this.state.local}`} className="mt-2 flex gap-2">
        <div className="flex gap-2">
          <button data-testid="local" className="bg-slate-200 px-2" onClick={() => this.setState((s) => ({ local: s.local + 1 }), () => trace("setStateDone", "life", this.state.local))}>local</button>
          <button data-testid="force" className="bg-slate-200 px-2" onClick={() => this.forceUpdate(() => trace("forceDone", "life"))}>force</button>
        </div>
        <div className="h-6 bg-fuchsia-300" style={{ width: 20 + this.props.n * 20 + this.state.local * 10 }} />
      </div>
    );
  }
}
class Pure extends PureComponent<{ label: string }> {
  render() {
    trace("render", "pure", this.props.label);
    return <div data-testid="pure" className="mt-2 h-6 w-32 bg-lime-300">{this.props.label}</div>;
  }
}
export const steps = [{ click: "next" }, { click: "local" }, { click: "force" }, { click: "next" }, { click: "next" }, { click: "next" }, { click: "drop" }];
export default function Case() {
  const [n, setN] = useState(0);
  const [on, setOn] = useState(true);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div className="flex gap-2">
        <button data-testid="next" className="bg-slate-200 px-2" onClick={() => setN(n + 1)}>next</button>
        <button data-testid="drop" className="bg-slate-200 px-2" onClick={() => setOn(false)}>drop</button>
      </div>
      {on && <Life n={n} />}
      <Pure label={n < 2 ? "low" : "high"} />
    </div>
  );
}
