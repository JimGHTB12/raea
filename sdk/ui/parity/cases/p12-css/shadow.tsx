// A box shadow (Tailwind's shadow-lg: two layered shadows).
// proves: tw.utility.shadow-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="card" className="mt-4 ml-4 h-24 w-48 rounded-lg bg-white shadow-lg" />
    </div>
  );
}
