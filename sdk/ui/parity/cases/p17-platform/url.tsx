// URL and URLSearchParams: parsing and serializing (case, default ports, dot segments, percent-encoding, IPv4, IDNA-free
// hosts), relative resolution, the setters, searchParams kept in step with search, canParse and parse.
// proves: api.URL, api.URL.URL, api.URL.href, api.URL.origin, api.URL.protocol, api.URL.host, api.URL.hostname, api.URL.port, api.URL.pathname, api.URL.search, api.URL.hash, api.URL.username, api.URL.password, api.URL.searchParams, api.URL.toJSON, api.URL.toString, api.URL.canParse_static, api.URL.parse_static, api.URLSearchParams, api.URLSearchParams.append, api.URLSearchParams.delete, api.URLSearchParams.get, api.URLSearchParams.getAll, api.URLSearchParams.has, api.URLSearchParams.set, api.URLSearchParams.sort, api.URLSearchParams.toString, api.URLSearchParams.size, api.URLSearchParams.entries, wintertc.URL, wintertc.URLSearchParams
import { useEffect } from "react";
import { trace } from "parity";
export const steps = [];
const parts = (u: URL) => [u.href, u.origin, u.protocol, u.username, u.password, u.host, u.hostname, u.port, u.pathname, u.search, u.hash];
export default function Case() {
  useEffect(() => {
    for (const s of [
      "HTTP://User:Pa%20ss@EXAMPLE.com:80/a/./b/../c?q=1 2#frag ment",
      "https://example.com:443",
      "https://example.com:8443/x/y/",
      "http://192.168.0.1:8080/",
      "http://0x7f.1/",
      "ws://chat.example/socket",
      "file:///Users/me/a%20b.txt",
      "mailto:someone@example.com",
      "data:text/plain,hi there",
      "https://example.com/été?é=è#à",
      "http://example.com/a\\b",
      "blob:https://example.com/uuid",
    ])
      trace("parse", s, parts(new URL(s)));
    const base = "https://example.com/dir/page.html?x=1#top";
    for (const r of ["other.html", "../up", "/root", "//cdn.example/lib.js", "?y=2", "#h", "", ".", "./a/b/../c"]) trace("relative", r, new URL(r, base).href);
    for (const bad of ["", "http://", "http://exa mple.com", "https://ex:99999/", "nope"]) trace("canParse", bad, [URL.canParse(bad), URL.parse(bad) === null]);
    try {
      new URL("/relative");
    } catch (e) {
      trace("throws", "relative", (e as Error).name);
    }
    const u = new URL("https://example.com/a?b=1&c=2&b=3");
    trace("params", "read", [u.searchParams.get("b"), u.searchParams.getAll("b"), u.searchParams.has("c"), u.searchParams.size]);
    u.searchParams.append("d", "x y&z");
    u.searchParams.delete("c");
    trace("params", "write", [u.search, u.href]);
    u.search = "?k=v";
    trace("params", "search-set", [u.searchParams.get("k"), u.searchParams.size]);
    u.hash = "sec 1";
    u.pathname = "/p q/r";
    u.port = "8080";
    u.hostname = "Other.Example";
    trace("setters", "after", parts(u));
    u.port = "443";
    u.protocol = "http";
    trace("setters", "default-port", [u.href, u.port]);
    const p = new URLSearchParams("a=1&b=%20x+y&c&=e&a=2");
    trace("usp", "parse", [...p.entries()]);
    p.set("a", "3");
    p.sort();
    trace("usp", "set-sort", p.toString());
    trace("usp", "from", [new URLSearchParams({ x: "1", y: "é !*'()~" }).toString(), new URLSearchParams([["k", "v"], ["k", "w"]]).toString()]);
    trace("json", "toJSON", JSON.stringify({ u: new URL("https://a.example/b") }));
  }, []);
  return <div data-testid="root" className="h-full w-full bg-white p-4"><div data-testid="box" className="h-8 w-32 bg-teal-300" /></div>;
}
