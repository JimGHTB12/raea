// sdk/r3f's parity cases drawn by three.js (docs/r3f.feature R3F8): #/r3f?dir=in&case=basic draws an R3F scene in R3F's
// own <Canvas>; #/r3f?dir=out&case=basic draws a raea composition in a <Canvas> through <Raea>; dir=src draws the same
// composition from the R3F source the source export wrote for it. Each steps its clock to
// the case's frame (1/fps a step, as raea's <R3F> does), waits for what it loads, draws once and says "drawn".
import { Canvas, advance } from "@react-three/fiber"
import { useEffect, useState } from "react"
import * as THREE from "three"
import { INPUTS, OUTPUTS, SIZE } from "./fixtures"
import type { ComponentType } from "react"
import { Raea } from "@raea/r3f/output/Raea"
import { loadShapes } from "@raea/r3f/three/geometry"
// the shape module tools/shape-wasm.sh builds into sdk/r3f/wasm, served by tools/serve.ts at /wasm/
const SHAPES = "/wasm/raea_shape_web.js"

function Stepper({ frame, onDone }: { frame: number; onDone: () => void }) {
  useEffect(() => {
    let live = true
    void (async () => {
      // out of React's commit: <Raea> renders its composition with flushSync inside useFrame
      await new Promise((r) => setTimeout(r, 0))
      for (let i = 0; i <= frame; i++) advance(i / SIZE.fps, true)
      // what loads later (models, pictures, the shape module): wait for it, then draw that frame again
      const m = THREE.DefaultLoadingManager as unknown as { itemsLoaded?: number; itemsTotal?: number }
      for (let k = 0; k < 40 && live; k++) {
        await new Promise((r) => setTimeout(r, 100))
        if ((m.itemsLoaded ?? 0) >= (m.itemsTotal ?? 0) && k > 5) break
      }
      advance(frame / SIZE.fps + 1e-6, true)
      if (live) onDone()
    })()
    return () => { live = false }
  }, [])
  return null
}

/** `sources`: the output cases as R3F source, each its module's <Scene> (the source export's, which tools/serve.ts writes
 * into test/parity/gen and hands to the page) */
export default function R3FParityPage({ sources = {} }: { sources?: Record<string, ComponentType<{ frame?: number }>> }) {
  const q = new URLSearchParams(location.hash.split("?")[1] ?? "")
  const dir = q.get("dir") ?? "in", name = q.get("case") ?? "basic"
  const [note, setNote] = useState("drawing")
  const [ready, setReady] = useState(dir !== "out")
  useEffect(() => {
    if (dir === "out") void loadShapes(SHAPES).then(() => setReady(true), (e) => setNote(`error: ${e}`))
  }, [])
  const done = () => setNote("drawn")
  const style = { width: SIZE.width, height: SIZE.height }
  if (dir === "in") {
    const c = INPUTS[name]
    return (
      <div style={{ background: "#000", ...style }}>
        <Canvas frameloop="never" dpr={1} gl={{ preserveDrawingBuffer: true }} style={style} {...(c.canvas as object)}>
          {c.scene()}
          <Stepper frame={c.frame} onDone={done} />
        </Canvas>
        <div className="font-mono" style={{ position: "fixed", left: 0, top: SIZE.height + 4, color: "#fff" }}>{note}</div>
      </div>
    )
  }
  const c = OUTPUTS[name]
  if (dir === "src") {
    const S = sources[name]
    return (
      <div style={{ background: "#000", ...style }}>
        <Canvas frameloop="never" dpr={1} shadows gl={{ preserveDrawingBuffer: true }} style={style}>
          <S frame={c.frame} />
          <Stepper frame={c.frame} onDone={done} />
        </Canvas>
        <div className="font-mono" style={{ position: "fixed", left: 0, top: SIZE.height + 4, color: "#fff" }}>{note}</div>
      </div>
    )
  }
  return (
    <div style={{ background: "#000", ...style }}>
      {ready && (
        <Canvas frameloop="never" dpr={1} shadows gl={{ preserveDrawingBuffer: true }} style={style}>
          <Raea component={c.component} frame={c.frame} width={SIZE.width} height={SIZE.height} fps={SIZE.fps} durationInFrames={c.frame + 1} />
          <Stepper frame={c.frame} onDone={done} />
        </Canvas>
      )}
      <div className="font-mono" style={{ position: "fixed", left: 0, top: SIZE.height + 4, color: "#fff" }}>{note}</div>
    </div>
  )
}
