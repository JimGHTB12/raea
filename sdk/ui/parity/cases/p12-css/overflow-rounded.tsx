// overflow-hidden with rounded corners clips what's in it to the rounded padding box: a full-bleed child, a child
// under a border (clipped at the border's inner curve), a picture-like gradient, and a faded one. The box's own
// shadow ring and border stay outside the clip.
// proves: css.properties.overflow, css.properties.border-radius
export const steps = [];
const fill = "absolute inset-0";
export default function Case() {
  return (
    <div data-testid="root" className="grid h-full w-full grid-cols-2 gap-6 bg-white p-6">
      <div data-testid="plain" className="relative h-32 overflow-hidden rounded-3xl bg-slate-200">
        <div className={`${fill} bg-rose-500`} />
      </div>
      <div data-testid="bordered" className="relative h-32 overflow-hidden rounded-[28px] border-8 border-sky-300 bg-slate-200">
        <div className={`${fill} bg-indigo-600`} />
      </div>
      <div data-testid="ring" className="relative h-32 overflow-hidden rounded-2xl bg-black" style={{ boxShadow: "0 0 0 4px rgb(16,185,129), 0 12px 24px -6px rgba(0,0,0,0.5)" }}>
        <div className="absolute -left-4 -top-4 h-40 w-[110%]" style={{ background: "linear-gradient(135deg, rgb(250,204,21), rgb(220,38,38))" }} />
      </div>
      <div data-testid="faded" className="relative h-32 overflow-hidden rounded-[40px] bg-slate-200 opacity-60">
        <div className={`${fill} bg-emerald-600`} />
        <div className="absolute left-2 top-2 h-12 w-12 bg-amber-400" />
      </div>
    </div>
  );
}
