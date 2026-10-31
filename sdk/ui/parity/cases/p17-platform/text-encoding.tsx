// TextEncoder and TextDecoder (UTF-8 both ways, astral and invalid sequences, fatal, BOM, streaming, encodeInto,
// UTF-16LE), and atob/btoa with their errors.
// proves: api.TextEncoder, api.TextEncoder.encode, api.TextEncoder.encodeInto, api.TextEncoder.encoding, api.TextDecoder, api.TextDecoder.decode, api.TextDecoder.fatal, api.TextDecoder.ignoreBOM, api.TextDecoder.encoding, wintertc.TextEncoder, wintertc.TextDecoder, wintertc.atob, wintertc.btoa
import { useEffect } from "react";
import { trace } from "parity";
export const steps = [];
export default function Case() {
  useEffect(() => {
    const enc = new TextEncoder();
    const s = "héllo €😀";
    const b = enc.encode(s);
    trace("enc", "bytes", [enc.encoding, [...b]]);
    trace("enc", "lone-surrogate", [...enc.encode("a\ud800b")]);
    const into = new Uint8Array(5);
    const r = enc.encodeInto("€ab€", into);
    trace("enc", "into", [r.read, r.written, [...into]]);
    const dec = new TextDecoder();
    trace("dec", "round", [dec.decode(b) === s, dec.encoding, dec.fatal, dec.ignoreBOM]);
    trace("dec", "invalid", dec.decode(new Uint8Array([0xff, 0x41, 0xe2, 0x82, 0x41, 0xf0, 0x9f, 0x98])));
    trace("dec", "bom", [dec.decode(new Uint8Array([0xef, 0xbb, 0xbf, 0x41])), new TextDecoder("utf-8", { ignoreBOM: true }).decode(new Uint8Array([0xef, 0xbb, 0xbf, 0x41])).length]);
    try {
      new TextDecoder("utf-8", { fatal: true }).decode(new Uint8Array([0xc3]));
      trace("dec", "fatal", "no throw");
    } catch (e) {
      trace("dec", "fatal", (e as Error).name);
    }
    const st = new TextDecoder();
    const parts = [new Uint8Array([0xe2, 0x82]), new Uint8Array([0xac, 0x21])];
    trace("dec", "stream", [st.decode(parts[0], { stream: true }), st.decode(parts[1], { stream: true }), st.decode()]);
    trace("dec", "utf16", new TextDecoder("utf-16le").decode(new Uint8Array([0x68, 0, 0x69, 0])));
    trace("dec", "labels", [new TextDecoder("UTF8").encoding, new TextDecoder("latin1").encoding]);
    try {
      new TextDecoder("nope");
    } catch (e) {
      trace("dec", "bad-label", (e as Error).name);
    }
    trace("b64", "btoa", [btoa("hello"), btoa(""), btoa("ÿþ")]);
    trace("b64", "atob", [atob("aGVsbG8="), atob(" aGVs bG8 "), atob("aGVsbG8")]);
    for (const bad of [() => btoa("€"), () => atob("a")]) {
      try {
        bad();
      } catch (e) {
        trace("b64", "error", [(e as DOMException).name, e instanceof DOMException]);
      }
    }
  }, []);
  return <div data-testid="root" className="h-full w-full bg-white p-4"><div data-testid="box" className="h-8 w-32 bg-amber-300" /></div>;
}
