// iframe-script.tsx's frame: plain DOM, no React. It draws its own origin, says hello to its parent, and answers a
// message with what it got.
const box = document.createElement("div");
box.style.cssText = "margin: 8px; padding: 4px 8px; background: #fde68a; font: 14px 'Parity Mono', monospace; color: #111";
box.textContent = `frame, origin ${self.origin}`;
document.body.appendChild(box);
addEventListener("message", (e) => {
  const line = document.createElement("div");
  line.style.cssText = "margin: 0 8px; font: 14px 'Parity Mono', monospace; color: #1e3a8a";
  line.textContent = `got ${e.data} from ${e.source === parent ? "the parent" : "elsewhere"}`;
  document.body.appendChild(line);
  parent.postMessage({ reply: `pong:${e.data}` }, "*");
});
parent.postMessage({ hello: true, origin: self.origin }, "*");
