// Params: values over time, ramps and signals.
import { Time, ToneAudioNode, ctx, seconds } from "./all";

// ---- params -------------------------------------------------------------------------------------------------------

/** A Tone Param or Signal: a value (set before the first frame: the graph's; after: an event) and its ramps. */
export class Param {
  constructor(readonly owner: ToneAudioNode, readonly key: string[], public readonly units = "number") {}
  get value(): number {
    let o: any = this.owner.options;
    for (const k of this.key) o = o?.[k];
    return typeof o === "number" ? o : 0;
  }
  set value(v: number) {
    if (ctx.graph === null) {
      let o: any = this.owner.options;
      for (const k of this.key.slice(0, -1)) o = o[k] ??= {};
      o[this.key[this.key.length - 1]] = v;
    } else this.ramp(v, 0, ctx.now);
  }
  protected ramp(v: number, over: number, at: number, exp = false) {
    // before the graph is fixed, a change at time 0 is the graph's value; later ones are events
    if (ctx.graph === null && at <= 0 && over <= 0) {
      this.value = v;
      return this;
    }
    ctx.emit({ n: this.owner.id, t: at, e: "param", p: this.key[this.key.length - 1], val: v, ramp: over, ...(exp ? { curve: "exp" } : {}) } as any);
    return this;
  }
  setValueAtTime(v: number, time?: Time) {
    return this.ramp(v, 0, seconds(time));
  }
  linearRampToValueAtTime(v: number, time: Time) {
    const end = seconds(time);
    return this.ramp(v, Math.max(0, end - ctx.now), ctx.now);
  }
  exponentialRampToValueAtTime(v: number, time: Time) {
    const end = seconds(time);
    return this.ramp(v, Math.max(0, end - ctx.now), ctx.now, true);
  }
  rampTo(v: number, rampTime: Time = 0.1, startTime?: Time) {
    return this.ramp(v, seconds(rampTime, 0), seconds(startTime));
  }
  linearRampTo(v: number, rampTime: Time, startTime?: Time) {
    return this.rampTo(v, rampTime, startTime);
  }
  exponentialRampTo(v: number, rampTime: Time, startTime?: Time) {
    return this.ramp(v, seconds(rampTime, 0), seconds(startTime), true);
  }
  targetRampTo(v: number, rampTime: Time, startTime?: Time) {
    return this.rampTo(v, rampTime, startTime);
  }
  setTargetAtTime(v: number, startTime: Time, timeConstant: number) {
    return this.ramp(v, timeConstant * 3, seconds(startTime));
  }
  cancelScheduledValues(_t?: Time) {
    return this;
  }
  cancelAndHoldAtTime(_t?: Time) {
    return this;
  }
  getValueAtTime(_t: Time) {
    return this.value;
  }
}

export const Signal = Param;
