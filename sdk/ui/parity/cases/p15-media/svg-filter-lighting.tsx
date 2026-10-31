// SVG lighting: feDiffuseLighting and feSpecularLighting on a bumped surface (a blurred alpha as its height), lit by
// feDistantLight, fePointLight and feSpotLight, each composited back over its shape; and a CSS filter: url() naming
// one of them on an HTML box.
// proves: svg.elements.feDiffuseLighting, svg.elements.feDiffuseLighting.in, svg.elements.feDiffuseLighting.surfaceScale, svg.elements.feDiffuseLighting.diffuseConstant, svg.elements.feSpecularLighting, svg.elements.feSpecularLighting.in, svg.elements.feSpecularLighting.surfaceScale, svg.elements.feSpecularLighting.specularConstant, svg.elements.feSpecularLighting.specularExponent, svg.elements.feDistantLight, svg.elements.feDistantLight.azimuth, svg.elements.feDistantLight.elevation, svg.elements.fePointLight, svg.elements.fePointLight.x, svg.elements.fePointLight.y, svg.elements.fePointLight.z, svg.elements.feSpotLight, svg.elements.feSpotLight.x, svg.elements.feSpotLight.y, svg.elements.feSpotLight.z, svg.elements.feSpotLight.pointsAtX, svg.elements.feSpotLight.pointsAtY, svg.elements.feSpotLight.pointsAtZ, svg.elements.feSpotLight.specularExponent, svg.elements.feSpotLight.limitingConeAngle, svg.elements.feComposite.operator, svg.elements.feComposite.k1, svg.elements.feComposite.k2, svg.elements.feComposite.k3, svg.elements.feComposite.k4, css.properties.filter.svg_elements
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full bg-white p-4">
      <svg data-testid="svg" width={760} height={360}>
        <defs>
          <filter id="l-diffuse-distant" x="0" y="0" width="100%" height="100%">
            <feGaussianBlur in="SourceAlpha" stdDeviation={6} result="bump" />
            <feDiffuseLighting in="bump" surfaceScale={6} diffuseConstant={1.1} lightingColor="#fff7e6" result="lit">
              <feDistantLight azimuth={225} elevation={40} />
            </feDiffuseLighting>
            <feComposite in="lit" in2="SourceGraphic" operator="arithmetic" k1={1} k2={0} k3={0} k4={0} />
          </filter>
          <filter id="l-diffuse-point" x="0" y="0" width="100%" height="100%">
            <feGaussianBlur in="SourceAlpha" stdDeviation={5} result="bump" />
            <feDiffuseLighting in="bump" surfaceScale={5} diffuseConstant={1} lightingColor="#ffffff" result="lit">
              <fePointLight x={60} y={40} z={60} />
            </feDiffuseLighting>
            <feComposite in="lit" in2="SourceGraphic" operator="in" />
          </filter>
          <filter id="l-specular-distant" x="0" y="0" width="100%" height="100%">
            <feGaussianBlur in="SourceAlpha" stdDeviation={4} result="bump" />
            <feSpecularLighting in="bump" surfaceScale={5} specularConstant={0.9} specularExponent={20} lightingColor="#ffffff" result="spec">
              <feDistantLight azimuth={235} elevation={45} />
            </feSpecularLighting>
            <feComposite in="spec" in2="SourceAlpha" operator="in" result="specIn" />
            <feComposite in="SourceGraphic" in2="specIn" operator="arithmetic" k1={0} k2={1} k3={1} k4={0} />
          </filter>
          <filter id="l-specular-spot" x="0" y="0" width="100%" height="100%">
            <feGaussianBlur in="SourceAlpha" stdDeviation={3} result="bump" />
            <feSpecularLighting in="bump" surfaceScale={4} specularConstant={1.2} specularExponent={12} lightingColor="#bae6fd" result="spec">
              <feSpotLight x={20} y={20} z={120} pointsAtX={90} pointsAtY={90} pointsAtZ={0} specularExponent={4} limitingConeAngle={30} />
            </feSpecularLighting>
            <feComposite in="spec" in2="SourceAlpha" operator="in" result="specIn" />
            <feComposite in="SourceGraphic" in2="specIn" operator="arithmetic" k1={0} k2={1} k3={1} k4={0} />
          </filter>
        </defs>
        <g filter="url(#l-diffuse-distant)"><rect x={20} y={20} width={160} height={140} rx={24} fill="#f97316" /></g>
        <g filter="url(#l-diffuse-point)"><circle cx={290} cy={90} r={70} fill="#22c55e" /></g>
        <g filter="url(#l-specular-distant)"><rect x={400} y={20} width={160} height={140} rx={70} fill="#6366f1" /></g>
        <g filter="url(#l-specular-spot)"><rect x={590} y={20} width={150} height={140} rx={12} fill="#0f172a" /></g>
        <g filter="url(#l-diffuse-distant)"><text x={20} y={300} fontSize={96} fontWeight={900} fill="#be123c">LIGHT</text></g>
      </svg>
    </div>
  );
}
