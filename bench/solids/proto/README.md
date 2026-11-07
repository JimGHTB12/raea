# solids prototype (10-27)

The scratch prototype behind docs/handoff/solids.md ("Prototype"): a relations layer over today's manifold API
(`rel.ts`), the object bench's hidden checks (`checks.ts`), relation-style and coordinate-style objects with red-team
variants (`objects.ts`), labelled parts into the ECS world with each label checked as a claim (`world.ts`), the run
(`bench.ts`) and a sheet coloured by material label (`sheet.tsx`). TS only, run on the engine; nothing here is built
into raea yet. The lane turns it into the SDK shape and the bench proper.

    raea run bench/solids/proto/bench.ts                       # every object through its checks; ends "bench ok"
    raea render bench/solids/proto/sheet.tsx --frame 0 -o sheet.png
