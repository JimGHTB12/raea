// A class component: state, setState and a lifecycle method.
// proves: react.components.Component
import { Component } from "react";
import { trace } from "parity";
class Counter extends Component<{}, { n: number }> {
  state = { n: 0 };
  componentDidMount() { trace("didMount", "counter"); }
  componentDidUpdate() { trace("didUpdate", "counter", this.state.n); }
  render() {
    return (
      <div>
        <button data-testid="inc" className="bg-slate-200 px-2" onClick={() => this.setState({ n: this.state.n + 1 })}>inc</button>
        <span data-testid="n" data-value={String(this.state.n)} className="ml-2">{this.state.n}</span>
      </div>
    );
  }
}
export const steps = [{ click: "inc" }];
export default function Case() {
  return <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900"><Counter /></div>;
}
