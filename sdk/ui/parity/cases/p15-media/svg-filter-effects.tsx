// The other SVG filter primitives: feTurbulence (both types, stitched), feMorphology (dilate, erode), feConvolveMatrix
// (an emboss kernel), feDisplacementMap driven by turbulence, feComponentTransfer (table, discrete, linear, gamma),
// feBlend modes, feTile and feImage.
// proves: svg.elements.feTurbulence, svg.elements.feTurbulence.baseFrequency, svg.elements.feTurbulence.numOctaves, svg.elements.feTurbulence.seed, svg.elements.feTurbulence.stitchTiles, svg.elements.feTurbulence.type, svg.elements.feMorphology, svg.elements.feMorphology.in, svg.elements.feMorphology.operator, svg.elements.feMorphology.radius, svg.elements.feConvolveMatrix, svg.elements.feConvolveMatrix.order, svg.elements.feConvolveMatrix.kernelMatrix, svg.elements.feConvolveMatrix.divisor, svg.elements.feConvolveMatrix.bias, svg.elements.feConvolveMatrix.edgeMode, svg.elements.feConvolveMatrix.preserveAlpha, svg.elements.feDisplacementMap, svg.elements.feDisplacementMap.in, svg.elements.feDisplacementMap.in2, svg.elements.feDisplacementMap.scale, svg.elements.feDisplacementMap.xChannelSelector, svg.elements.feDisplacementMap.yChannelSelector, svg.elements.feComponentTransfer, svg.elements.feComponentTransfer.in, svg.elements.feFuncR, svg.elements.feFuncG, svg.elements.feFuncB, svg.elements.feFuncA, svg.elements.feBlend, svg.elements.feBlend.in, svg.elements.feBlend.in2, svg.elements.feBlend.mode, svg.elements.feTile, svg.elements.feTile.in, svg.elements.feImage, svg.elements.feImage.href, svg.elements.feColorMatrix.type, svg.elements.feColorMatrix.values
export const steps = [];
const DOT = "data:image/svg+xml," + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><circle cx="10" cy="10" r="7" fill="#e11d48"/></svg>');
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full bg-white p-4">
      <svg data-testid="svg" width={760} height={400}>
        <defs>
          <filter id="e-noise" x="0" y="0" width="100%" height="100%"><feTurbulence type="fractalNoise" baseFrequency={0.04} numOctaves={3} seed={7} stitchTiles="stitch" /><feColorMatrix type="saturate" values="0" /></filter>
          <filter id="e-turb" x="0" y="0" width="100%" height="100%"><feTurbulence type="turbulence" baseFrequency="0.02 0.08" numOctaves={2} seed={3} /></filter>
          <filter id="e-dilate"><feMorphology in="SourceGraphic" operator="dilate" radius={3} /></filter>
          <filter id="e-erode"><feMorphology in="SourceGraphic" operator="erode" radius="2 1" /></filter>
          <filter id="e-emboss"><feConvolveMatrix order={3} kernelMatrix="-2 -1 0 -1 1 1 0 1 2" divisor={1} bias={0} edgeMode="duplicate" preserveAlpha /></filter>
          <filter id="e-warp" x="-10%" y="-10%" width="120%" height="120%">
            <feTurbulence type="turbulence" baseFrequency={0.05} numOctaves={2} seed={2} result="t" />
            <feDisplacementMap in="SourceGraphic" in2="t" scale={18} xChannelSelector="R" yChannelSelector="G" />
          </filter>
          <filter id="e-transfer">
            <feComponentTransfer in="SourceGraphic">
              <feFuncR type="table" tableValues="0 0.2 1" />
              <feFuncG type="discrete" tableValues="0 0.5 1" />
              <feFuncB type="gamma" amplitude={1.2} exponent={0.6} offset={0} />
              <feFuncA type="linear" slope={0.8} intercept={0.1} />
            </feComponentTransfer>
          </filter>
          <filter id="e-blend-multiply" x="0" y="0" width="100%" height="100%"><feFlood floodColor="#38bdf8" result="f" /><feBlend in="SourceGraphic" in2="f" mode="multiply" /></filter>
          <filter id="e-blend-screen" x="0" y="0" width="100%" height="100%"><feFlood floodColor="#7c3aed" result="f" /><feBlend in="SourceGraphic" in2="f" mode="screen" /></filter>
          <filter id="e-tile" x="0" y="0" width="100%" height="100%">
            <feImage href={DOT} x={0} y={0} width={20} height={20} result="dot" />
            <feTile in="dot" />
          </filter>
        </defs>
        <rect x={10} y={10} width={170} height={110} filter="url(#e-noise)" />
        <rect x={200} y={10} width={170} height={110} filter="url(#e-turb)" />
        <text x={400} y={80} fontSize={56} fontWeight={800} fill="#0f172a" filter="url(#e-dilate)">raea</text>
        <text x={590} y={80} fontSize={56} fontWeight={800} fill="#0f172a" filter="url(#e-erode)">raea</text>
        <g filter="url(#e-emboss)"><circle cx={70} cy={200} r={55} fill="#f59e0b" /><rect x={30} y={190} width={90} height={18} fill="#b45309" /></g>
        <g filter="url(#e-warp)"><rect x={170} y={150} width={160} height={110} fill="none" stroke="#0f172a" strokeWidth={10} /><line x1={170} y1={205} x2={330} y2={205} stroke="#db2777" strokeWidth={8} /></g>
        <rect x={360} y={150} width={170} height={110} fill="url(#e-grad)" filter="url(#e-transfer)" />
        <linearGradient id="e-grad"><stop offset="0" stopColor="#ef4444" /><stop offset="0.5" stopColor="#22c55e" /><stop offset="1" stopColor="#3b82f6" /></linearGradient>
        <circle cx={620} cy={205} r={55} fill="#fde047" filter="url(#e-blend-multiply)" />
        <circle cx={90} cy={330} r={55} fill="#16a34a" filter="url(#e-blend-screen)" />
        <rect x={200} y={280} width={200} height={100} filter="url(#e-tile)" />
      </svg>
    </div>
  );
}
