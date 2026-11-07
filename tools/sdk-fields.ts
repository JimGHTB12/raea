// Field coverage (docs/control.feature FC0b): for each Bevy type the SDK says it reaches by hand (a tier-1 row of
// tools/sdk-coverage), its reflected fields (raea-bevy-sdk --schema) against the props of the exports that set it: the
// tagged export and the Bevy-named one of the same short name in sdk/render/src/bevy3d. A field counts as named when a
// prop of either has its name. Named isn't proven: the round trip (set the prop, read the component back over BRP) is
// FC0b's second half. Props with no field of that name are listed too: each is a convention (a raea name or unit) or a
// prop for another component.
//
//   bun tools/sdk-fields.ts ROWS.tsv SCHEMA.json > docs/control/fields.tsv
// (sdk/shell holds the classic compiler, whose API this reads; sdk/raea's is TS 7's native one.)
// ROWS.tsv is `tools/sdk-coverage --rows`; SCHEMA.json is raea-bevy-sdk --type T... --schema's output.
import ts from "../sdk/shell/node_modules/typescript/lib/typescript.js";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const [rowsPath, schemaPath] = process.argv.slice(2);
const root = join(import.meta.dir, "..");
const schema = JSON.parse(readFileSync(schemaPath, "utf8")) as { types: Record<string, { fields?: { name: string }[] }> };

const dirs = ["sdk/render/src/3d", "sdk/render/src/bevy3d", "sdk/render/src"];
const files = dirs.flatMap((d) => readdirSync(join(root, d)).filter((f) => /\.tsx?$/.test(f) && !f.endsWith(".d.ts")).map((f) => join(root, d, f)));
const program = ts.createProgram(files, { jsx: ts.JsxEmit.ReactJSX, target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext, moduleResolution: ts.ModuleResolutionKind.Bundler, strict: true, skipLibCheck: true, noEmit: true, allowImportingTsExtensions: true });
const checker = program.getTypeChecker();

// every exported function's first parameter's property names, by "file:Name" and by Name within bevy3d
const props = new Map<string, Set<string>>();
for (const f of files) {
  const sf = program.getSourceFile(f);
  if (!sf) continue;
  const mod = checker.getSymbolAtLocation(sf);
  if (!mod) continue;
  const rel = f.slice(root.length + 1);
  for (const sym of checker.getExportsOfModule(mod)) {
    const decl = sym.valueDeclaration;
    if (!decl || !(ts.isFunctionDeclaration(decl) || ts.isVariableDeclaration(decl))) continue;
    const t = checker.getTypeOfSymbolAtLocation(sym, decl);
    const sig = t.getCallSignatures()[0];
    const p = sig?.getParameters()[0];
    if (!p) continue;
    const pt = checker.getTypeOfSymbolAtLocation(p, decl);
    const names = new Set(checker.getApparentType(pt).getProperties().map((s) => s.getName()).filter((n) => n !== "children" && n !== "key"));
    props.set(`${rel}:${sym.getName()}`, names);
  }
}

const out = ["# type\tfields\tnamed\tmissing\textra props (no field of that name)\texports"];
let total = 0, named = 0;
for (const line of readFileSync(rowsPath, "utf8").split("\n")) {
  const [area, item, , tier, reach] = line.split("\t");
  if (area !== "bevy" || tier !== "1") continue;
  // a tuple field (Mesh3d(Handle), RenderLayers(mask)) is set by the part itself or its children, not a prop
  const fields = schema.types[item]?.fields?.map((f) => f.name).filter((n): n is string => typeof n === "string");
  const short = item.replace(/<.*$/, "").split("::").pop()!;
  const exports = [...(reach ?? "").matchAll(/(\w+) \(([^)]+)\)/g)].map((m) => `${m[2]}:${m[1]}`);
  for (const f of ["sdk/render/src/bevy3d/core.tsx", "sdk/render/src/bevy3d/lights.tsx", "sdk/render/src/bevy3d/materials.tsx", "sdk/render/src/bevy3d/mesh2d.tsx", "sdk/render/src/bevy3d/raytracing.tsx"]) {
    if (props.has(`${f}:${short}`)) exports.push(`${f}:${short}`);
  }
  const have = new Set(exports.flatMap((e) => [...(props.get(e) ?? [])]));
  if (!fields) {
    out.push([item, "-", "-", "no schema (an asset or a non-struct type)", "", exports.join(" ")].join("\t"));
    continue;
  }
  const hit = fields.filter((f) => have.has(f));
  const extra = [...have].filter((p) => !fields.includes(p)).sort();
  total += fields.length;
  named += hit.length;
  out.push([item, fields.length, hit.length, fields.filter((f) => !have.has(f)).join(" "), extra.join(" "), exports.join(" ")].join("\t"));
}
out.push(`# all\t${total}\t${named}\t${total - named} fields no typed prop names`);
console.log(out.join("\n"));
