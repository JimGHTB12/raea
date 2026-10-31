// CSS filter: url(#id) on HTML elements, naming SVG <filter>s in the page: a blur, a red-blue swap (feColorMatrix, in
// linearRGB as SVG's primitives are), an feDropShadow, an offset copy merged under the element, a url() followed by
// a function (grayscale, in linear light after a linearRGB filter, as Chrome chains them), a function before a url()
// (sepia, in sRGB), and a url() to nothing, which leaves its element unfiltered.
// proves: css.properties.filter
export const steps = [];

const Card = ({ f, label }: { f: string; label: string }) => (
  <div className="flex w-[120px] flex-col items-center gap-2">
    <div data-testid={label} className="flex h-[70px] w-[90px] items-end rounded-lg bg-rose-600 p-2" style={{ filter: f }}>
      <div className="h-6 w-6 rounded-full bg-sky-500" />
    </div>
  </div>
);

export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-6">
      <svg width={0} height={0} className="absolute">
        <defs>
          <filter id="u-blur">
            <feGaussianBlur stdDeviation="3" />
          </filter>
          <filter id="u-swap">
            <feColorMatrix type="matrix" values="0 0 1 0 0  0 1 0 0 0  1 0 0 0 0  0 0 0 1 0" />
          </filter>
          <filter id="u-shadow" x="-30%" y="-30%" width="160%" height="160%">
            <feDropShadow dx="6" dy="6" stdDeviation="3" floodColor="#0f172a" floodOpacity="0.6" />
          </filter>
          <filter id="u-echo" x="-10%" y="-10%" width="140%" height="140%">
            <feOffset in="SourceGraphic" dx="12" dy="10" result="moved" />
            <feColorMatrix in="moved" type="saturate" values="0" result="grey" />
            <feMerge>
              <feMergeNode in="grey" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>
      </svg>
      <div className="flex flex-wrap gap-6">
        <Card label="blur" f="url(#u-blur)" />
        <Card label="swap" f="url(#u-swap)" />
        <Card label="shadow" f="url(#u-shadow)" />
        <Card label="echo" f="url(#u-echo)" />
        <Card label="then-grey" f="url(#u-swap) grayscale(1)" />
        <Card label="sepia-first" f="sepia(1) url(#u-swap)" />
        <Card label="missing" f="url(#u-none)" />
      </div>
    </div>
  );
}
