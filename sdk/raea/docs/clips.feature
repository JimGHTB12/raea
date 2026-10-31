Feature: One motion database per body
  Set by Shane, 2026-10-22: "there should be no limiting anything... leverage all of these animations into a single
  system where we have total control of rigs and limbs and motion trajectories and events... why separate them if a
  human can do it its part of the same engine"; "this is why i said we could use HDC and vector embeddings to search";
  "this is a job for GPU kernel".

  Before this, every scene baked a hand-picked subset of captures into JSON and matched over it
  (examples/character/parkour.json: 57 clips, mostly runs), so the item and grip scenes walked on run frames: crouched
  starts, and 25 cm dips on slow carries. The database replaces the subset with everything each body has retargeted
  (the kid: 1,566 clips, 932,797 frames, 1,865,594 with their mirror images), and a wrong pick is fixed by
  conditioning (tags, masks, per-limb control), never by shrinking the library.

  Code: sdk/raea/src/clips.ts (the file, the tag vocabulary, the HDC encoder, the library it loads into, the
  search backends), sdk/raea/tools/library.ts `db` (the build), sdk/raea/src/matching.ts (perform's pluggable
  search and the steps' conditioning). Bench: sdk/raea/bench/character/clips.ts. The GPU search (the GPU kernels:
  the GPU kernel kernels, the bun:ffi cdylib and the native calls) was deleted in the overhaul (10-27): nothing registered its
  calls. Its rules and GPU kernel results are in this file's history.

  Rule: The evaluation (first: is HDC / embedding search on GPU kernel worth it here?)
    Shane, 2026-10-22: "now the main focus should be the comprehensive system and seeing if hdc/embedding GPU kernel lookup
    and calculation is helpful here". Run on the kid's full database (~1.87M frames mirrored) by
    bench/character/clips.ts; the table and the verdict are recorded in this file's Results section.
    Scenario: The methods compared
      Then A is the JS exact brute force (today's nearest()): the reference
      And B is the exact quantised cost as a GPU kernel kernel, one thread a frame, reduced to the best 64-frame blocks, the
        top K blocks reranked exactly (and its TS integer twin, kernel-ts, bit-exact)
      And C is HDC: each frame's features as a 256-bit hypervector (Levels plus role binding), Hamming on GPU kernel for a
        top-k prefilter, then the exact CPU rerank
      And D is HDC with the tags bound into the same hypervector, against B or C with the tags as a separate bitmask
        filter: does putting the semantics in the vector help or hurt?
      And E is a cheap CPU index, no GPU: a coarse-to-fine scan (every 8th frame scored, the best coarse frames' 64-frame
        neighbourhoods reranked) and IVF over k-means clusters
    Scenario: The metrics, for each method
      Then recall of A's best frame among the candidates at K = 1, 4, 16, 64 (blocks or frames, said which)
      And the rank of A's pick in the method's order and the cost gap (the method's pick's exact cost over A's)
      And milliseconds a search, warm and cold; for GPU kernel upload, compute and readback apart
      And memory (RSS, GPU buffers)
      And perform's end-to-end time for the grip scenes and a walk-wait-carry-stop
      And the motion's quality on those performances: the grips bench, the head-dip and crouch check, the lint's skate
        and pop counts, so approximate picks that visibly hurt the motion show
    Scenario: Semantic retrieval
      Given steps that want carry, look-around, suitcase and box
      Then for each method, the share of returned frames from the right captures (CMU 70_*, 79_25, 111_36, 113_26,
        115_*, 69_69 for carries; 40_10 and the "looking around" clips for looks)
      And the same with the tags as a bitmask filter, as bound HDC roles, and as both
    Scenario: The verdict
      Then the Results section ends with the method or combination the engine uses for offline renders and for live
        play at 30 fps, backed by the table

  Rule: The file (lib/BODY/motion.db, built offline by `bun tools/library.ts db SKELETON.json LIB`)
    Scenario: What a frame carries
      Given a body's retargeted library (LIB/index.json, LIB/cmu/*.json, LIB/100style/*.json)
      When the db command builds LIB/motion.db
      Then every frame of every clip in the index has a row (the unmirrored frames; the mirror images are derived on
        load, exactly, because a mirror is a permutation and a negation of the normalised, quantised columns)
      And each row holds: the 27 motion features, normalised over the database's matchable frames and quantised to
        i16 at 1/128 of a unit; the 21 STATE measurements as i16 z-scores at 1/128; the ground frame (x, z, yaw, f32)
        for playback and steering; a flag byte (has a route second, foot contacts L and R, crouched, moving); the hips'
        height over its clip's median, x1000; the hips' ground speed, cm/s; two words of tag bits; and an HDC
        hypervector of 256 bits
      And each clip in the header has its name, source, clip file, CMU free-text label or 100STYLE label and move, its
        tags and its frame range
      And the file is a 16-byte magic and length, a JSON header, then 64-byte aligned little-endian sections: it is
        read with one file read into typed arrays (bun) or mmapped (Rust), never imported as JSON into a bundle
    Scenario: Poses are referenced, not copied
      Then a row's playback pose is its clip file and frame: the loader reads a clip file the first time one of its
        frames plays, stood on a level floor (levelClip) exactly as the features were measured
      And the features are measured from the levelled clips, so the feet's heights in a query and in the database agree
    Scenario: Tags
      Then CMU's free-text descriptions are normalised into a fixed vocabulary of at most 64 tags (clips.ts TAGS):
        walk, run, jog, turn, jump, hop, sit, stand, idle, carry, suitcase, box, heavy, pick, place, reach, look-around,
        climb, stairs, crouch, sneak, dance, fight, throw, catch, drink, chore, push, pull, balance, hurt, styled,
        backward, sideways, uneven, playground, swing, stretch, lie, fall, and 100STYLE's own (style, its move FW or ID)
      And each frame also gets measured tags: moving (over 0.15 m/s across the second about it), still, fast (over
        2 m/s: a run's), crouched (hips under 0.92 of the clip's median or coming to it within 12 frames)
      And `clips.ts tagsOf(label)` is pure and tested on every CMU label: "carry 5.5lb suitcase" is carry and
        suitcase, "moving heavy box" is carry, box and heavy, "walk forward and pick up object, carry back object" is
        walk, pick and carry
    Scenario: HDC hypervectors
      Then a frame's hypervector binds its motion with its tags in one 256-bit word string (8 u32 words): bit b is the
        majority over the frame's 27 features, each bound to its role (feature d's random bit r(d, b) xor its level's
        flip mask, level = the feature's value in 16 steps over +-4 units: the Levels construction) and over its tags,
        each its role's random bit h(64 + t, b); ties go to a fixed tie vector
      And every random bit is an integer hash (look_proj.k's), so the TS encoder, the K CPU interpreter and the WGSL
        kernel give the same bits; the stored words are checked against the kernel's on 10,000 sampled frames: 0 differ
      And one query asks for both "moves like this" and "is a carry": its hypervector is the query's features bound the
        same way bundled with the wanted tags' role vectors
    Scenario: Size and load time on the kid
      Then LIB/motion.db is under 170 bytes a frame (the kid's 2,703 clips, 1,488,590 frames: 234 MB, 157 B a frame)
      And it builds in under 2 minutes (48 s on the kid, 2.2 GB peak)
      And bun loads it into a matchable library (mirror images included) in under 3 s and 2 GB RSS (0.1 s read, 0.46 s
        library, 1.34 GB), against 2.1 s and 3 GB for the JSON tables of fewer clips (scratchpad/full/proto.ts)

  Rule: Perform's pluggable search
    Scenario: Backends
      Then a MotionLibrary may carry a search backend (lib.search); perform calls it instead of brute force wherever it
        searched (the start, each search every few frames), and reranks what it returns with the same cost
      And the JS brute force stays the reference and the small-library path: a library without a backend searches
        as before, and every bench below runs unchanged on its own library
      And the backends are: "js" (brute force), "kernel-ts" (the kernel's integer scoring in TS, slow), HDC, the
        coarse scan and IVF
    Scenario: Conditioning by tags and masks
      Then a step may say `tags: { want?: string[]; forbid?: string[]; cost?: Record<string, number> }`: frames lacking
        a wanted tag or having a forbidden one are never chosen, and each present tag's cost is added
      And a spec may say the same for every step (spec.tags), a step's own adding to it
      And the defaults from a step's kind, on a database library: a wait forbids crouched, a walk under 1.8 m/s forbids
        fast and crouched, a carry walk (items.ts: a thing held in both hands, or heavy) wants carry
    Scenario: Per-limb control (the legs-and-upper split, built: matching.ts limbs.upper, items.ts)
      Then a walk or wait may say `limbs: { upper: { tags?, hands? } }`: the legs, hips and way play the step's match,
        and the upper body (the spine up, the arms, the head) its own, searched with the legs' query (so its stride is
        theirs) among frames with the upper's tags as well as the step's, kept unless something is clearly better, and
        blended into, between and out of through its own inertialized switches; an action takes the whole body
      And `hands` asks where each hand should be in the body's frame, matched against the database's hands section
        (U: each frame's hands in its ground frame, mirrored exactly), so a one-hand carry plays a suitcase's hand low
        at the side rather than a box's before the chest
      And items.ts asks it of every step while a thing is held (the upper from carry captures, a one-hand thing's hand
        where interact's hold puts it, a glass's before the waist; a heavy thing takes the legs from carries too)
      And a walk at walking pace on a database wants the "walk" tag unless it asks for tags of its own (the legs had
        played pantomimes and chores, whose long strides put the thigh into a mug carried at the side)
      And on a library without tags (parkour.json) the step plays whole, as before
      And a walk at walking pace holding a thing in one hand says `limbs.clear` (its half width, items.ts), and frames
        whose arm swing would bring it within that and a centimetre of the thigh on its side or the waist over the
        next half second (matching.ts swingClearance) aren't matched, on any library; the database stores each frame's
        clearance per hand (K, version 3, mm), parkour.json's are worked out from its poses. Grips 22/30 to 25/30 on
        parkour.json, and over the databases kid 9 to 10, the man 5 to 6, the woman 5 to 6; not on runs (the kid's
        carry over the hurdle skated) nor waits (the kid's patient look skated)
      And not yet: the upper body held from a frame or masked for the items' IK; a hand's facing as well as its place
        (tried: the palm's and fingers' way in U, scored 1 - cos; it did not help, kid 9/10 to 7-8/10, the man 5/10
        either way, so the wrist the capture gives is not what puts the thumb into a can)

  Rule: The proof
    Scenario: Walks no longer dip onto run frames
      When the grip scenes (examples/character/grip-scenes.ts) are performed over the kid's full database
      Then no walk or carry plays a frame tagged fast, and the hips' lowest point mid-walk is less far below standing
        than on parkour.json's library in every scene, and under 11 cm (measured: 4.2-10.1 cm, against 12.7-16.9)
      And no walk plays a frame stood on something (tagged raised: a planted foot 6 cm up, or the hips 8% over
        standing; "clean" and "Unspecified" captures of stairs and ladders were walked on before it)
    Scenario: Heavy carries pick carry captures
      Then the bigball and bigbox carries and a 4 kg carry walk play frames of carry captures (CMU 70_* suitcases,
        79_25 moving a heavy box, 111_36 and 113_26 walk and carry, 115_* box pick-ups, 69_69) for at least 80% of
        their walking frames
    Scenario: A walk, a wait, a carry and a stop
      Then bench/character/clips.ts performs each over the full database and prints what it played, its dips, its
        foot slide and its time
    Scenario: The bars do not move
      Then `bun bench/character/parkour.ts --body all` passes as many levels with no more flags than before (kid 5/5;
        man fails 3, 4, 5 with 2, 3, 7 flags; woman fails 2-5 with 1, 3, 4, 2): parkour stays on its own library until
        the unified one matches or beats it on that bench
      And `bun bench/character/grips.ts --body all` passes at least 22/30
      And `bun test conformance/` in sdk/raea passes 309

  Rule: Results (2026-10-22, the kid: 2,703 clips, 2,977,180 frames mirrored; bench/character/clips.ts)
    Search, over 300 real queries sampled from the 2,000 the scenes' perform runs asked (ms are TS twins on one core
    until the GPU kernel row, which is the GPU kernels's):
      | method                              | K            | recall of A's best | cost gap mean / max | ms a search |
      | A JS brute force (reference)        | all          | 1                  | 0 / 0               | 25.1        |
      | B exact quantised, best blocks      | 1 block      | 0.987              | 0.0002 / 0.021      | 106.6 (TS)  |
      | B                                   | 4-64 blocks  | 1                  | 0 / 0               | 106.6 (TS)  |
      | C HDC motion + tag bitmask          | 64 frames    | 0.31               | 1.90 / 73.5         | 215 (TS)    |
      | C                                   | 4,096        | 0.67               | 0.13 / 3.7          |             |
      | C                                   | 16,384       | 0.79               | 0.07 / 3.7          |             |
      | D HDC with the tags bound in        | 64           | 0.013              | 21.3 / 942          | 199 (TS)    |
      | D                                   | 16,384       | 0.38               | 3.9 / 440           |             |
      | E coarse scan, every 8th frame      | 32 coarse    | 0.76               | 0.06 / 1.0          | 24.2        |
      | F IVF, 256 k-means clusters         | 2 probes     | 0.92               | 0.011 / 0.58        | 4.0         |
      | F                                   | 8 probes     | 1                  | 0 / 0               | 16.5        |
    The median rank of A's best in C's Hamming order is 449, in D's 43,331: a 256-bit Levels-and-roles code of 27
    features keeps the coarse shape of a pose and its route, not the fine distances matching turns on.

    Semantic retrieval, 40 walk queries each asking for a tag (share of the returned frames, and of the picks, from
    the right captures):
      | tag         | A filter | B filter K=16 returned / picks | C + filter | D bound, no filter | D bound + filter |
      | carry       | 0.725    | 0.646 / 0.725                  | 0.58/0.675 | 0.003 / 0          | 0.003 / 0.571    |
      | suitcase    | 1        | 0.998 / 1                      | 1 / 1      | 0.003 / 0          | 0.003 / 1        |
      | box         | 0.1      | 0.162 / 0.1                    | 0.32 / 0.1 | 0.001 / 0          | 0.001 / 0        |
      | look-around | 0.85     | 0.833 / 0.85                   | 0.82/0.85  | 0.019 / 0          | 0.019 / 0.769    |
    Untagged, every method returns 0-0.5% right frames. Binding a tag into the hypervector does nothing for retrieval:
    one tag among 27 features' votes barely moves the majority, so it is drowned, and with the filter on as well it
    only costs recall. A tag belongs in the bitmask. (Box is low because CMU's box captures are pick-ups, not walks.)

    The motion, each scene performed with each method as perform's search (P is today's library, parkour.json):
      | scene                  | P mid-walk dip | A/B dip | P skate (cm, all frames) | A/B skate | carry share A/B | grip clip frames P / A |
      | ball                   | 15.8           | 4.2     | 464                      | 180       |                 | 20 / 43 (both fail)    |
      | mug                    | 15.8           | 5.9     | 398                      | 211       |                 | 2 / 36                 |
      | can                    | 15.8           | 5.9     | 398                      | 211       |                 | 2 / 19                 |
      | shoebox                | 16.9           | 4.4     | 459                      | 235       |                 | 2 / 1                  |
      | ball-look              | 15.8           | 10.1    | 464                      | 206       |                 | 20 / 57                |
      | bigball                | 15.9           | 7.7     | 501                      | 136       |                 | 0 / 0                  |
      | bigbox (carry)         | 12.7           | 7.7     | 825                      | 172       | 0.85            | 4 / 4                  |
      | walk-wait-carry-stop   | 5.3            | 8.4     | 261                      | 133       | 0.99            |                        |
    B's performances are A's frame for frame (recall 1 at 16 blocks). C, D and E change the picks and the motion
    moves with them, both ways: C's ball-look dips 10.1 cm and clips 76 frames, D's carries pick carry frames
    0.80-0.82 of the time against A's 0.85-0.99. The grip clipping is not the search's: it follows which walk's arm
    swing the hand rides (C, a worse search, clips less on mug and can), which is what per-limb control is for.
    perform takes 1.8-3.8 s a scene with A over the whole database (the tag filters prune most frames before costing).

    GPU kernel (the GPU kernels, an M5 Pro, the kid's 2,977,180 frames, 200 recorded queries): the TS twin, Rust, the GPU kernel
    interpreter and the GPU are bit-exact on every frame of every query. The exact search is 4.43 ms warm median (upload
    0.07, compute 3.96, readback 0.29), 8.2 ms p95, 7.1 ms cold, with 548 MB of GPU buffers resident in 1.8 s. Rust on
    the CPU takes 54 ms (1 thread) or 18.6 ms (6 threads). Through bun:ffi with the TS rerank it is 11.1 ms a search,
    recall 1 at 4 blocks or more, and the grip scenes perform in 0.6-1.6 s against A's 2.1-3.7 s, frame for frame
    the same.

    The man and the woman (their databases built the same way, 2,703 clips each): the database's walks dip 7.6-11.9 cm
    mid-walk against parkour.json's 8.4-17.6, with about half the skate, and carry walks pick carry captures 0.97 of
    the time. Two scenes are worse: the woman's glass walk dips 22.8 cm (against 18.3) and her big box 16.4 (8.3).

    Grips over the database with per-limb control (clip frames; a scene passes at 10 or fewer):
      | body  | ball | pencil | glass | mug | can | shoebox | ball-look | can-look | bigball | bigbox | pass |
      | kid   | 24   | 0      | 5     | 3   | 3   | 9       | 9         | 0        | 1       | 6      | 9/10 |
      | man   | 4    | 0      | 7     | 23  | 44  | 40      | 16        | 63       | 0       | 6      | 5/10 |
      | woman | 11   | 0      | 8     | 37  | 41  | 2       | 4         | 55       | 0       | 38     | 5/10 |
    19/30 against parkour.json's 22/30. The kid's were ball 43, mug 36, can 19, ball-look 57, can-look 45 before it.
    The adults' clips are mostly the thumb 0.7-1.6 cm into a light can or mug, which rides about 0.8 of the capture's
    arm: next is the fingers' closing, pulled onto the surface as well as stopped at it (SAGA), with a few millimetres
    of give for the skin (ContactOpt). So the grip scenes and parkour stay on parkour.json for now.

    The verdict:
    - Nearest-neighbour picks: an exact search, never HDC. The 256-bit code ranks A's best 449th (C) and 43,331st (D);
      even 16,384 Hamming candidates recall 0.79. HDC as a prefilter costs motion quality and saves no time over an
      exact scan that takes 4.4 ms on the GPU.
    - Conditioning: tags as a bitmask in the scoring pass (want, forbid, cost), never bound into the hypervector.
    - Offline renders: B, the exact quantised kernel on GPU kernel, reranked; the same picks as A at about half the time end
      to end. A, the JS brute force, stays the reference and the path where no GPU is open.
    - Live play at 30 fps: B. At 4.4 ms warm (8.2 ms p95) a search, one search a character every 6 frames fits
      several characters in a frame's 33 ms. F (IVF, 256 clusters, 8 probes, recall 1 here, 16.5 ms in TS) is the CPU
      fallback for one character.
    - Before parkour and the grips move onto the database: the adults' grips (the thumb), and parkour's own bench on
      the database.
