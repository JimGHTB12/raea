@spec
Feature: Direction that holds: one world of facts, findings and rules, so a person's notes become checks and a project is built in an order that can't skip modelling, blocking or review
  Shane, 11-10, after a two-minute film went through eleven passes and was abandoned ("the video is dead in the
  water"): "i think the problem is you have kept reworking the same scenes again and again, the whole video is now
  broken ... i will have to make sure we instead of even fixing understand". Then: "take time to put all of the
  pieces together from my notes into one large spec and feature. how do we truly fix all of these at the core."

  His findings, in his words:
    1. Notes. "the inability to understand my notes as imperative thoughts that need further design. When I leave a
       note on a comment, I could be encapsulating a lot of things. Why is it that these notes become kind of like not
       seen or not heard, or when I say something in one round, it doesn't get heard in another ... The first step of
       taking these notes should be converting them to basically some sort of TypeScript test that we can validate
       against. What is it that they're asking for? Can we create a rule? Can we create a controller? How do
       controllers become rules? These are things that need to be inked into our skills."
    2. Looks. "we have an overreliance on our shaders that we wrote for sample projects in raea ... the paint and
       other looks ... are really hurting us because the point should be that you design your own shader for every
       project, for every video, or if you're saving it, you're saving it as a controller across the project, or a
       component across the project".
    3. Components. "this project was bound to fail because we didn't build a component library at the very beginning
       that was highly organized. Every prop should be a component. Every component has props that we animate. Those
       are rigs. We need to have a much more structured approach".
    4. Blocking. "blocking first was actually really good ... we got the best results when we didn't even worry about
       camera angles and we just thought about blocking ... make sure that we understand clipping and rigs within
       blocking scenes ... the computer jumps on the chair. In almost every scene, the chair and the computer would
       then be misaligned or broken, and it would be clipping through the chair. And then we wouldn't get the same
       telemetry back about clipping. Where did that telemetry go? It used to be there. Those kind of controllers
       should help with clipping. How we measure clipping, what components are attached".
    5. Medium. "I kept saying I want 2D animated text with occlusion, and you kept putting in 3D text extrusion ... if
       I want 3D text, I'll say 3D text. If I want 2D text, I'll say 2D text ... you're not over relying on oh it's a
       3D project so it's 3D text ... we are compositing layers and good content leverages 2D and 3D in composites".
       And of the lyrics: "think of how a lyric video would be and we want all lyrics like that".
    6. Sense and detail. "the desks are all facing the wrong direction and why is there kitchen props on the desk?
       That makes no sense. We need semantic layers ... we should be very focused on the taxonomies of these
       procedural elements ... we need to over detail the scenes, meaning detail that the camera might not ever see
       ... If the goal is to create a cityscape outside ... graffiti and billboards and cars ... medians and
       sidewalks and people and windows and fire escapes ... plastering a billboard over a street because I said I
       want a billboard there, it just doesn't make any sense. Things have to look good."
       Of the places it flew through: "all the worlds you keep featuring are like voxel low poly. i dont want that.
       i want highly defined and highly modeled scenes and world of varying style ... you have put zero time into
       any world modeling work i asked for."
    7. Review. "there's clearly a lack of checking any of the check systems, like how we match up, first of all
       camera angle consistency ... The visualization of a shot is not working because there's so much movement going
       on ... we're losing a lot of quality in the movement that the stills aren't showing you. Same with the audio
       since you can't listen to the audio, you're not seeing any of that."
    8. One system. "check files and controllers and the cli results and data should all be intertwined somehow. maybe
       theres a better way to use the ecs system for all of these at least in format and standard but checks should
       apply to raea as a whole not just bevy".

  What the agent on that film did, as the failure each part below is tested against:
    - It patched each note and logged a row with a still. Nothing kept checking it. Asks made in one round (2D type,
      modelled places, real meshes, not primitives) were re-read or dropped in a later one. It never restated what a
      note held, so a note holding three asks got one answered.
    - Asked for "our own shader", it added options to the stock paint look.
    - Its props were one-off meshes inside scene files, placed by loops with no idea what a prop is or where it goes.
      Fourteen "worlds" were loops of spheres, boxes and cones.
    - raea printed its clipping lines on every render and it filtered them out of every run (`grep -v clipping`).
      Its own rider-and-seat check measured the seat's angle, not whether the rider was in the chair's back.
    - It judged from single frames. A value that changes over a shot was right in every still and frozen in the
      film (a camera column the continuous render doesn't refresh): only the full render showed it.
    - It wrote eight check scripts inside the project (motion, framing, seat, light, clipping, lyric placement, frame
      jumps, cue sheet). Each had its own output; none fed another; three were left with flags unread.
    - It never heard the mix.

  What raea already has (read 11-10; each is built on, none is replaced), and the gap in each:
    | what                                   | where                                              | the gap                                                                 |
    | an entity-component world, TS or engine | sdk/app/src/ecs.ts (raea/ecs: spawn, find, relations, resources, systems; JsStore under bun) | used by games only; a composition isn't read into one |
    | a composition stepped with no engine   | runtime/export/src/fed.rs scene_frames (each frame's 3D items), runtime/export/src/scene.rs | 3D items only; the page's text and layers aren't in it |
    | an edit as rows and keyed columns      | core/program (ir, eval, timeline)                  | the renderer's own; nothing queries it for checks                        |
    | what each frame shows, as laid out     | `raea render --manifest` (runtime/export/src/lib.rs write_manifest): each text's words and box, each video's source and time, the 3D's clipping, the sounds heard, the layers | written only when asked for; read by `raea check` alone; its clipping list is a second copy of what was printed |
    | parts that clip, float, sink           | systems/contact (every render), `raea scene lint`   | findings are printed text or loose JSON `{check, level, says}`           |
    | an export's floor                      | apps/check (`raea check`: text, picture, flash, sound; `--quick`; `--taste`) | its own `Issue {rule, p, at, says}`; exports only     |
    | a person's standing rules with checks  | core/brief/src/taste.rs (`Rule { text, check }`), the taste loop (docs/craft-system.md J) | six closed forms, all about an export's text and sound; a rule is proposed only when a correction repeats; nothing per project or per note |
    | notes as tasks                         | review.feature R3 (to do, working, done), a pin's `part` | "done" is the agent's word; nothing proves it or keeps it       |
    | notes mapped to a verb                 | skills/skills/raea/craft/verbs                      | one verb a note; a note that holds several asks loses the rest           |
    | a plan frozen before building          | core/brief/src/plan.rs (parts, levels, counts, `seen_from`), `raea solid info --expect` | one object at a time; not a place, not a project   |
    | rooms that know what things are        | raea/room (`layRun` roles, `faceToward` from a model's measured front, `checkLayout`, `checkDecor`) | interiors against walls only; a set built any other way gets none of it |
    | placement by relation                  | scene.feature (`<Place on>`; surfaces by name and a contact model per asset are "next") | still next                                    |
    | a shader over a whole scene            | shaders.feature SH4 (`<SurfaceShader>`)             | the skill offers `<Look mode="paint">` first (3d.md); nothing says a project should have its own |
    | every drawn thing's layer key and mask | review.feature R2, systems/layers (page built, 3D next) | made for selecting in the player; not used to mask one layer by another |
    | text behind a subject                  | goal.feature Track G: "depth-based text layering (G5, measured; not rendered yet)" | not rendered                             |
    | measures per frame, per cut, over time | goal.feature Track G's catalogue (previz columns)   | for footage edits; reported, not enforced; not run on a 3D or React piece |
    | controllers                            | core/brief/src/controller.rs, apps/workspace/src/controllers.rs | settings only (choice, list, agents, workflow); no rule, no check |
    | components registered and linted       | projects.feature PJ8 (`raea project scan`)          | a component is "a .tsx a composition imports"; it carries no meaning     |
    | the order of work                      | skills/skills/raea/craft/ROUTER.md                  | storyboard and camera map before anything is blocked; no component or blocking step |
    | the next step                          | apps/workspace/src/next.rs                          | says "render ...: stills to look at first"                               |
    | skill evals                            | skills.feature SK7                                  | no task takes notes over two rounds                                      |

  The rules this plan sets:
    1. One world. A project is read into one world of entities and components in raea/ecs's form, the same under bun
       and in the engine: its layers, text, parts, shots, cuts, cues; its notes; its rules; and every finding. A check
       is a system over that world. A rule is a resource in it. A command's result is what it added to it. Nothing in
       the format is 3D's or Bevy's: a caption, a cut and a cue are entities a check asks about as it does a mesh.
    2. One finding. Every result raea gives has one shape, is kept (in the render's manifest and the project's record),
       and can be asked for again. No command only prints.
    3. A note is an ask to be designed. It is first restated (everything it holds, its medium, what it doesn't ask
       for) and given a check that fails now. Nothing changes before that.
    4. A check is code where the ask can be measured, and a named clip the person signs where it can't; it says
       which. A row with a still is neither.
    5. Rounds hold. Every earlier round's checks run in every later round; a regression names the note it breaks.
    6. A rule climbs: a note's check, kept for the project, becomes a project rule (a controller); kept across
       projects, the person's taste. It is the same check at each level, moved, never rewritten.
    7. The order of work is components, blocking, look, cameras, sound, each reviewed before the next. A later stage
       is refused on a failing earlier one.
    8. Every prop is a component whose typed props are its rig, and which says what it is and where it belongs.
       Nothing is placed by code that can't read that. Modelling is a stage, with a floor, and comes first.
    9. A project's look is its own, kept in the project. raea's sample looks are examples to read.
    10. The medium of an element is the person's word for it. A piece is a composite of 2D and 3D layers.
    11. What moves is reviewed moving; what sounds is measured; what was not seen or heard is said.
    12. A finding is fixed or carried with its reason. An agent never filters raea's findings out of a run.
    13. A measure an agent can satisfy cheaply is found first: every kind of check ships with cheats that must fail.
    14. Each of these lands in the raea skill through raea-skills, and an eval shows an agent with only the skill
        follows it.

  Background:
    Given the raea skill is raea-skills' (the skills/ submodule); a change to it is a raea-skills PR and a pin bump
    And a project is a folder in raea's format (projects.feature), its workspace's controllers in controllers/
    And raea/ecs is the format: `spawn(components)`, `find({ with, without, where })`, relations as components,
      resources by name, a system `(world, frame) => void`, `world.at(frame)` the same whenever it's asked for

  # =====================================================================================================================
  Rule: Part 0. Decisions

    @done
    Scenario: DR0. The decisions this plan needs (decided 11-10; Shane can veto any line later)
      # Status 11-10: decided. Shane set the goal to build this file with "DR0 first (write my recommended decisions
      # into the file as decided; I can veto later)". Each line below is now what the later scenarios build on; a
      # veto changes the line here and the scenarios that read it, in one commit.
      Then the world is raea/ecs's (sdk/app/src/ecs.ts), grown, not a second store: the reading of a project into it
        is Rust (it reuses scene_frames and the page's layout), exposed to TS checks through the same Store calls
      And a finding's shape is `Finding { check, level: "error" | "warn" | "note", about: Entity[], frames: [a, b] |
        null, says, measure: { name, value, unit, limit } | null, fix: string | null, by: string }`, and the two
        ranks in use today map onto it (apps/check's P0 and P1 are errors, P2 a warn, P3 a note; lint's levels as
        they are)
      And a project's notes and their checks live in the project, in git: notes/<round>/<id>.ts, one file a note,
        `export default note({ words, from, pin, asks: [...] })` with each ask's check beside its words; no Markdown
      And a check is a TS function, typed from the world's components, run by `raea check` with no flag for where it
        came from; raea's own checks stay in Rust and are registered under the same names
      And a controller gains one field type, `rule`: `{ text, check, level }`, the check a name (one of raea's) with
        its values or a path to a project check; a project rule is a controller with rule fields
      And the taste file's six forms become six of raea's named checks (same names, same values), so
        `{ form: "text-above", value: 62 }` still reads and no taste file changes
      And the stages are recorded in .raea/project (`[stage] components = passed <sha>` ...), set by the stage's
        review, read by `raea next` and by the refusals
      And what the stock paint look gained on 11-10 (light as a gradient; worlds cut per pixel with styles; the
        camera columns it needs each frame) is taken out of systems/look into the worked example of a project look
        (DR24), not landed as part of the stock look
      And every scenario below gets its plan table (crate, file, lines, job) when it is claimed; the tables given
        here are for Parts 1 and 2, which everything else stands on

  # =====================================================================================================================
  Rule: Part 1. One world: a project read into raea/ecs, for every kind of piece (finding 8)

    @needs:DR0 @done
    Scenario: DR1. A composition's frame as a world, with no render
      # Status 11-10: built. `raea world` reads any composition's frames into raea-core-facts worlds, each entity's
      # Source from the code stamps. Real reads (release binary, M5 Pro): content/rome/film.tsx frame 900 is the 3D
      # layer (camera moving, parts measured, the caesar-bg model "not measured": raea can't read its triangles), four
      # cues with their sources, a picture at 7% and seven runs of laid-out type ("49 BC" 34 px, "The Dictator" 72 px,
      # the caption at the bottom); content/reel/edit.tsx at 314-318 shows the cut from shot noir-lamp (221-316) to the
      # next at 316; examples/games/breakout.tsx frame 60 has its 34 parts and "SCORE 10" at 60 px. Layout costs 0.33 s
      # on Rome's frame (0.39 s against 0.06 s with --no-layout) and 80 ms a frame on the reel (1.02 s against 0.21 s
      # for ten frames). Proved by 8 tests in
      # raea-runtime-world (a page read as shots, footage, cues and text in document order, a <style>'s CSS not read as
      # words; 3D as one layer with its camera's velocity and measured parts; a stamp's file, line and component; a
      # transcript's lines), 2 in raea-app-direction (one find across media; a bad query or range says what it takes)
      # and 1 in raea-page (letters set one a span read as their words). The cheat: a part's kind comes from its
      # shape, so a box its author named and propped as 3D text "SALE" is a mesh with no Text (scene.rs). Not seen:
      # content/anything/edit/scene.tsx throws its own "not a song map" error before any frame; the reel's flying
      # letters (v/build/text.tsx) are one solid a glyph, not Text3D, so they read as Parts with no Text, which is what
      # they are. Found on the way and fixed: the render's manifest joined a word set one letter a span as "4 9 B C".
      Given the plan
        | crate              | file                           | lines | job                                                                    |
        | raea-core-facts    | core/facts/src/components.rs   | 300   | an Item, a page box, a clip, a cue as components (the names and fields below) |
        | raea-core-facts    | core/facts/src/words.rs        | 40    | a transcript's words, as `raea check` reads them (its Word, moved here) |
        | raea-runtime-world | runtime/world/src/lib.rs       | 140   | a composition bundled and stepped to its frames, as a bake steps them (React only, nothing drawn), each frame read into a world |
        | raea-runtime-world | runtime/world/src/scene.rs     | 200   | the 3D: plates, the 3D layer with its camera and parts (measured from their triangles), 3D text, the 2D drawn over it |
        | raea-runtime-world | runtime/world/src/timeline.rs  | 330   | the page and timeline: layers in document order, shots (Sequences), clips, cues, lines (the project's words.json, <Speech>) |
        | raea-runtime-world | runtime/world/src/page.rs      | 60    | the page laid out with nothing drawn (BevyPage::lay_out) and read as the render's manifest reads it |
        | raea-runtime-world | runtime/world/src/source.rs    | 90    | Source from the stamps: file, line, the component named at the nearest use, the layer key |
        | raea-app-direction | apps/direction/src/world.rs    | 200   | `raea world`: the frame's world as text or JSON, or one query of it    |
      And three new crates hold Parts 1 and 2 (raea-cli and raea-runtime-export are at their 2,000-line limits, and
        raea-app-check is six lines under its own): core/facts, runtime/world, apps/direction
      And a project of any kind: a React video, a footage edit, a page, a game, a 3D scene, or several composited
      When `raea world EDIT.tsx [--composition ID] [--frame N | --at A-B|A,B] [--find JSON] [--no-layout] [--json]` runs
      Then the frame is one world: its 3D items, timeline, clips and cues read as `raea scene lint` reads a 3D frame
        today (React stepped, no engine, no GPU)
      And the page's text and boxes come from the page's layout, which before this only a render did (the manifest's
        "as laid out", which `raea check --quick` gets by rendering three stills): this scenario adds a layout-only
        pass (the render's own page synced and laid out by the main world's schedules, the render world's never run,
        so nothing is drawn); with `--no-layout`, or where the page can't be made, its text carries `needs: "render"`
      And its entities carry these components, each field measured, none typed by the author:
        | component  | on                               | fields                                                                  |
        | Source     | everything                        | file, line, the component's name and its use chain (review.feature R2a's key) |
        | Layer      | each composited layer             | medium: "2d-type" / "2d-graphic" / "3d" / "footage" / "page"; order; opacity; blend |
        | Text       | a run of type, 2D or 3D           | words, font, size px at 1080, box in the frame, extruded (depth), lines |
        | Part       | a 3D mesh or model                | component, tags, world box, transform, triangles, materials             |
        | Camera     | the frame's camera                | eye, look, lens or ortho height, near, far; its velocity to the next frame (its move) |
        | Shot       | a stretch between two cuts        | its name, from, to (a Sequence's box: <Sequence> stamps data-raea-end); the shot around it as its parent |
        | Clip       | a piece of footage on the timeline | source file and its hash, in, out, at; its words (the transcript's)     |
        | Cue        | a sound placed on the timeline    | source, at, gain, bus                                                   |
        | Line       | a sung or spoken line             | its words with their times (the project's words.json); a <Speech> line's text and start, `needs: "transcript"` for its times |
      And relations are components too (`Facts::relate`, `{ to }`): ChildOf is every entity's parent here; Rides, Holds,
        StandsOn and HangsFrom are set by DR16, Masks and MaskedBy by DR19, SungAt by DR19's lyric checks, CausedBy by DR21
      # Fixed in the code (DR1): the plan gave a Shot "its camera's move" and a Cue "its cause's entity". A world is one
      # frame, and a shot's whole move needs every frame of it: each frame's Camera carries its velocity instead, and a
      # range read (--at A-B) is the shot's move frame by frame. A sound's cause is nothing the tree says yet; DR21
      # gives a cue its cause, so CausedBy is its to set. A <Speech> line has no word times until it's transcribed.
      And a query spans media: "every Text whose Layer is 2d-type and that no Part masks during its Line" is one find
      And the same frame gives the same world whenever it's asked for, and a range of frames gives one a frame

    @needs:DR1
    Scenario: DR2. What only a render knows joins the same world
      Then every render writes its manifest (today only with --manifest), beside its output
      And the manifest is entities too, added to the frame's world when a check asks for them: the layer
        masks (review.feature R2b) as `Covers { layer, share }` between layers, each text's contrast against what is
        behind it, each shot's measured motion and luminance (goal.feature Track G's columns), the mix's loudness and
        each cue's measured onset
      And a check says whether it needs them (`needs: "render"`); `raea check` on a project with no render runs the
        rest and lists what waits for one

    @needs:DR1
    Scenario: DR3. One finding, for every command
      Given the plan
        | crate              | file                           | lines | job                                                                    |
        | raea-core-facts    | core/facts/src/finding.rs      | 160   | Finding, its JSON, its one-line text; the two old ranks mapped onto it |
        | raea-sys-contact   | systems/contact/src/found.rs   | 140   | the clipping check's and the lint's results as Findings about entities |
        | raea-app-direction | apps/direction/src/issue.rs    | 150   | raea check's Issue as Finding (rule -> check, p -> level, at -> frames) |
        | raea-app-direction | apps/direction/src/record.rs   | 220   | the project's record: findings by render and by commit, read back      |
        | raea-app-direction | apps/direction/src/findings.rs | 160   | `raea findings`                                                        |
      Then these write Findings and nothing else: the render's clipping check, `raea scene lint|light`, `raea check`
        (and --quick, --taste), `raea gen check`, `raea solid info --expect`, `raea project scan`, raea/room's
        checkLayout and checkDecor, `raea judge`
      And each finding names the entities it is about (the two parts that clip, the text that is cut off), so
        "everything found about this part" is a query
      And every render, video or `--at` stills, writes its findings into its manifest and prints their count last,
        errors first, whatever its output is piped through; `raea render` exits 1 on an error unless `--despite`
      And .raea/record/ keeps them by render and commit; `raea findings [--about NAME] [--check ID] [--open]
        [--since RENDER] [--json]` reads them, and says what was fixed, what is new and what is carried since
      And `raea next` reads from it (not from a check.json's modified time, as next.rs does now)

    @needs:DR1,DR3
    Scenario: DR4. A check is a system; raea's own are written the same way
      Given the plan
        | crate              | file                           | lines | job                                                                    |
        | raea-app-direction | apps/direction/src/systems.rs  | 300   | raea's checks registered by name with their values; a project's TS checks run beside them |
        | raea-core-facts    | core/facts/src/rule.rs         | 180   | Rule: text, check (a name and values, or a path), level; taste's Check as six of them |
      And sdk/raea/src/check.ts (about 220 lines): `check({ id, text, needs, level }, (world, frame) => ...)`,
        `found(...)`, and the components' types generated from core/facts (ts-rs, as core/brief's are)
      Then `raea check [PROJECT | EDIT.tsx | VIDEO.mp4] [--stage S] [--only ID] [--json]` runs every check that
        applies: raea's floor, the person's taste, the workspace's and the project's rules, the project's notes
      And a check is told once what it ranges over (`frames: "all" | "cuts" | "lines" | [..]`) and is run there; it
        finds entities, measures, and calls `found(...)`; it never prints
      And the same check text reads for a mesh and for a caption: `world.find({ with: ["Text", "Layer"] })`
      And proved in small on 11-10 (bench/direction/proto/world.ts, plain bun, raea/ecs as it is today, no engine):
        one world holding a 2D lyric layer, a 3D rider and seat, a footage cut and a sound cue; four checks as
        systems; findings and a three-ask note as entities; six cases and two cheats, all as expected
      And the proof that it isn't 3D's: five checks that import nothing of Bevy's, each with a cheat that fails it
        | check                                                                  | on                         |
        | a lyric line is 2D type, inside the frame, masked by the subject       | a React layer over footage |
        | a cut lands on a word's end                                            | a footage edit             |
        | a cue falls within a frame of its cause                                | a mix                      |
        | a caption clears the platform's button band                            | a vertical ad              |
        | a rider turns with its seat and is in no part of it                    | a 3D scene                 |

  # =====================================================================================================================
  Rule: Part 2. Notes become asks and checks, and rounds hold (finding 1)

    @needs:DR4
    Scenario: DR5. A note is restated before anything changes
      Given the plan
        | crate              | file                           | lines | job                                                                    |
        | raea-core-facts    | core/facts/src/note.rs         | 200   | Note: words, from, round, pin, asks; Ask: text, medium, about, not, check, proof |
        | raea-app-direction | apps/direction/src/notes.rs    | 320   | `raea notes take, ls, check, retire, keep`; a note as an entity in the world |
      And sdk/raea/src/note.ts (about 90 lines): `note({...})` for notes/<round>/<id>.ts
      And notes sent from the app (review.feature R3a: their words, their pin with its frame, place and part), or
        said in a session
      When `raea notes take` runs (the agent runs it first, on receiving them)
      Then each note is a file with the person's words verbatim and no asks yet, and the task reads "to restate"
      And the agent writes its asks: every separate thing the note holds, one ask each; the medium where the words
        name one (2D, 3D, flat, extruded, real, modelled, voxel); what it is about (the pin's part, found in the
        world, or a query); and `not`: what the note does not ask for, so a change keeps the rest
      And a note with one ask and more than one sentence of words is flagged for a second reading, and a note that
        asks for a redesign ("from scratch", "not the vibe", "redesign") is marked so: its asks say what is kept
      And the verbs (skills/skills/raea/craft/verbs) are per ask, not per note
      And the restated asks are posted back on the note in the app before work starts, where the person can correct
        one in a word; a correction is a new line of the note, kept

    @needs:DR5
    Scenario: DR6. Each ask gets a check that fails now, or a clip to sign
      Then an ask's check is a system (DR4) written in the note's file or named from raea's, and `raea notes check`
        must show it failing on the work as it stands before the work may change (a check that already passes is
        refused: it isn't measuring the ask)
      And each finding carries the ask that asked for it (`by`), so two asks that use one named check are told
        apart (the prototype counted both asks' findings against each until it did)
      And where the ask can't be measured, its proof is a clip: `proof: { shot | frames, sign: true }`; it passes only
        when the person marks it in the app (their name, the render's hash), and reopens when its source changes
      And the app's task badge (R3) reads: to restate, to check, failing, passing, signed; "done" is never typed by
        the agent
      And worked checks ship in the skill for the asks that recur, each with its cheat:
        | the ask, as a person says it                         | the check                                                     |
        | "the seat turns with the computer"                    | Rides: offset and turn from its socket every frame; no Covers of the rider by the seat's parts |
        | "2D text with occlusion"                              | Text.extruded is false; Layer.medium is 2d-type; MaskedBy the subject where they meet |
        | "desks face the wrong way", "kitchen props on a desk" | every Part's `meta.front` toward what it serves; no Part of kind X StandsOn kind Y           |
        | "hold this longer"                                    | the Shot is at least N seconds                                |
        | "no orbs anywhere"                                    | no Part of the named component in any frame                   |
        | "this is low poly", "more detail"                     | the component passes its detail floor at its nearest (DR14)   |
        | "the light isn't timed to the cut"                    | the lit share of the subject crosses its half within a frame of the cut |

    @needs:DR6
    Scenario: DR7. Rounds hold
      When `raea check` runs, or a render is asked for
      Then every round's notes run; one that passed in an earlier round and fails now is a regression, reported
        first, with its note's words and the commit that broke it (the record, DR3)
      And `raea render` of a composition whose project has a note to restate, to check, or regressed is refused,
        naming them, unless `--despite ID "why"`, which is written into the manifest and shown on the post
      And a note is retired only by the person (`raea notes retire ID --by NAME`, or in the app), and a redesign's
        note retires the checks it names as replaced, never silently

    @needs:DR7
    Scenario: DR8. A note's check climbs to a rule, and to taste
      Given the plan
        | crate              | file                           | lines | job                                                                    |
        | raea-core-brief    | core/brief/src/rule_field.rs   | 90    | a controller field of type `rule` (controller.rs's Kind gains it)      |
        | raea-app-workspace | apps/workspace/src/rules.rs    | 80    | rule fields read as checks; `raea notes keep ID --as project, workspace or taste` |
      Then `raea notes keep` moves a note's check to controllers/<id>.ts as a rule field (project or workspace), or
        to the person's taste as a rule; the note keeps a pointer; the check's text is not copied
      And the taste loop (docs/craft-system.md J) proposes this when a correction repeats, as it does now, and now
        for any check, not six forms
      And a rule's field (a limit, a list of kinds) is edited in the app's Controllers page like any field, and the
        check reads it: "how do controllers become rules" is this
      And `raea controller ls --rules` lists every rule that applies to a project and where it came from

    @needs:DR6
    Scenario: DR9. A check that can be gamed is found before it's trusted
      Then every named check ships with cheats that must fail it, run in CI: the ask met on one frame only; the part
        hidden, renamed, scaled to nothing or moved off frame; the text made 2D by a depth of 0.001; the rider
        detached from its seat; the check's own limit edited
      And an agent may not edit a note's check after it first failed, except to tighten it; a loosened check is a
        finding against the commit

  # =====================================================================================================================
  Rule: Part 3. The order of work, enforced (findings 3, 4 and 7)

    @needs:DR3
    Scenario: DR10. Five stages, each with its review, recorded
      Then the order in the skill's craft/ROUTER.md becomes: context; shape (the brief); storyboard (what happens,
        with no cameras); components; blocking; look; cameras; sound; evidence
      And each stage closes by its review (`raea stage review components` ...), which runs that stage's checks and
        makes what a person looks at (a sheet for components, clips for the rest); the person's yes, or the checks
        alone where the brief says so, records it in .raea/project
      And `raea next` names the open stage and its first failing check, and no longer says "stills to look at first"
      And starting a later stage's work on a failing earlier one is a finding at every render, not a refusal of the
        file save: a camera can be sketched early, but nothing with a camera can be posted

  # =====================================================================================================================
  Rule: Part 4. Components with meaning, and modelling with a floor (findings 3 and 6)

    @needs:DR10
    Scenario: DR11. A project begins with its component library
      Given `raea init --components`, or a project whose brief names what it holds
      Then components/ has one file for each prop, set piece and character: a default export with typed props, the
        animatable ones declared as its rig (`rig({ open: range(0, 1), turn: angle(), ... })`), and `meta`
      And `meta` says: what it is (a path in the project's taxonomy: furniture/desk, kitchen/kettle, street/sign);
        its real size; its up and front (measured from the model as raea/room's faceToward does, not typed); what it
        stands on, hangs from or sits in; what may stand on it; its sockets (a seat, a slot, a plug); how near it is
        seen (`seenFrom`, as core/brief's Plan has it)
      And `raea project scan` (PJ8) reads it; a mesh or model built inline in a scene file is a finding ("a prop
        outside components/"), with the move as its fix
      And `raea components sheet` renders each alone: three views, its rig at rest and at each extreme, its size
        against a person; the stage's review is this sheet

    @needs:DR11
    Scenario: DR12. Placement reads what things are, anywhere, not only against a room's walls
      Given raea/room (roles, measured fronts, checkLayout) and scene.feature's surfaces by name and contact model
        per asset, built here where still "next"
      Then `place({ what, on | in | against | along, facing, count, spacing, seed })` places by relation and kind in
        any set; it refuses a pairing the taxonomy doesn't allow, and turns each thing by its measured front
      And a `populate` of many (desks down a floor, cars on a street) takes rules, not coordinates: what serves what
        (a chair its desk, a desk its aisle), what belongs on what, how many, how they vary
      And the checks, as raea's named ones: every part with a front faces what it serves; nothing stands on a kind it
        can't; nothing floats or sinks (the lint); no two of a populated kind are identical beyond a share
      And the cases: an office of forty desks, none facing a wall, no kitchenware on one; a street whose billboard is
        on a roof or a gantry built to carry it

    @needs:DR11
    Scenario: DR13. A place has a written plan of what it holds, before it's built
      Given core/brief's Plan (an object's parts, levels and counts, frozen before building)
      Then a place (a room, a street, a world) has a plan of the same kind: its zones, what each holds by taxonomy,
        the counts, and the detail that makes it that place, including what a camera may never see
      And the plan is written from what such a place has (a street: kerbs, medians, pavements, crossings, signs,
        shopfronts, windows, fire escapes, wires, bins, wear, people, traffic), frozen, and the build is checked
        against it: a planned kind with no component, or a count unmet, is a finding
      And a thing added because a note asked for it ("a billboard there") goes through the plan: where such a thing
        stands, on what, among what

    @needs:DR11
    Scenario: DR14. A detail floor, and modelling is a stage
      Then `raea components check` measures each component at its `seenFrom`: the frame share it fills, its distinct
        parts, its triangles and edge detail against that share, its surface detail (textures, or shader detail), and
        whether it is one primitive or a few; under the floor is a finding, and the stage doesn't close
      And the floor is a rule (DR8) a project can raise, never an agent
      And how a component is made follows the skill's routing (built in code when exact, a permissive library, or
        generated), and a spend or a download is the person's yes, asked once for the stage with its quote
      And the stage's time is the largest in the plan the agent gives: a brief with twelve places says so before
        any is built, and offers fewer, finished

  # =====================================================================================================================
  Rule: Part 5. Blocking first, with rigs and clipping (finding 4)

    @needs:DR11
    Scenario: DR15. A blocking composition with no cameras
      Then `blocking.tsx` is the set and the cast on their marks through the piece's time, registered as its own
        composition, drawn from fixed views (top, front, side, three-quarter), never a shot's camera
      And `raea blocking check` needs no render: the clipping check on every frame, the lint, every relation (DR16),
        every mark reached when its line or beat says
      And it answers fast enough to be run on every save: a scene of 5,000 parts in under ten seconds a frame and
        a two-minute piece in under two minutes, measured in CI. Measured 11-10: `raea scene lint --frame 27 --json`
        on one 80-frame shot of the abandoned film (a few thousand parts) had given no answer after 7 minutes at
        3.4 GB and was stopped; whether it is slow or stuck there isn't known yet, and finding out is this scenario's
        first step
      And `raea stage review blocking` makes one clip a scene from the fixed views, with findings marked on the frames
      And the cameras stage reads positions from blocking and may not move the cast

    @needs:DR15,DR4
    Scenario: DR16. A relation is declared once and measured every frame
      Then `<Rides rider seat socket facing>`, `<Holds>`, `<SitsOn>`, `<PlugsInto>` say who is on what; the rider's
        place and turn come from the socket (the component's `meta`), so they can't drift apart
      And the check: the rider's offset and angle from its socket, in millimetres and degrees, every frame it holds;
        and no part of the rider inside any part of the seat (the clipping check, asked for that pair, without the
        share and depth allowances it gives parts at rest)
      And getting on and off are part of it: a path that passes through the seat's back or arm is a finding
      And the case that failed for a whole film: a computer on a swivel chair

    @needs:DR3
    Scenario: DR17. Clipping telemetry can't go missing, and says what is attached
      Then the clipping lines are Findings in the manifest and the record (DR3), counted last on every run
      And each names both parts' components, what each is attached to (ChildOf, Rides, StandsOn), and the frames
      And a part that passes through another on purpose carries `allowOverlap="why"`: the reason is required, and
        `raea findings --carried` lists them for the person
      And the skill says, in its core: never filter raea's findings out of a command's output

  # =====================================================================================================================
  Rule: Part 6. A look of the project's own (finding 2)

    @needs:DR10
    Scenario: DR18. The project's look is a file in the project, with its controller
      Given shaders.feature SH4 (`<SurfaceShader>`: one shader over every lit surface of a scene) and frame passes
      Then looks/<name>.wesl is the project's shading, and looks/<name>.ts its controller: the uniforms as fields
        (colours, ramps, widths), so a look is tuned in the app and animated like any rig
      And `<ProjectLook name>` puts it on a scene; the look stage's review is one scene from blocking under it, in a
        clip, beside the brief's references
      And `raea init` writes no `<Look mode>`; skills/skills/raea/3d.md and 3d/post.md present the frame looks as
        worked examples with their source, under "writing your project's look"
      And a stock look in a project with no look of its own is a finding at the look stage ("a sample look as the
        project's"), carried only by the person

    @needs:DR18
    Scenario: DR24. A worked project look, from what was written on 11-10
      Then the gradient light, the world cuts (a mesh drawn only inside its ball, in the main pass and the prepass)
        and the nine styles are an example project look under examples/, on SurfaceShader, with its controller
      And the stock paint look is as it was before 11-10; the camera columns a continuous render refreshes each frame
        (runtime/scene/src/live.rs cam_keyed) take a look's declared uniforms, not a list of names

  # =====================================================================================================================
  Rule: Part 7. Layers, and the medium asked for (finding 5)

    @needs:DR2
    Scenario: DR19. A 2D layer masked by what is in a 3D or footage layer
      Given systems/layers (each pixel's frontmost layer) and goal.feature's G5 (a person's matte and depth)
      Then `<Layer medium="2d-type" maskedBy={["hero", "desk"]} depth={z}>` composites flat type or graphics behind
        the named parts and in front of the rest: the mask is the engine's own, exact to the pixel, for a 3D scene
        from its depth and for footage from its matte
      And raea's text module (sdk/raea/src/text.ts, grown) has the lyric-video set: a line as words, each arriving on its sung time, with the motions a
        lyric video uses (slide, wipe by a mask, scale, track), all 2D
      And an element's medium is recorded on its Layer and its Text, so a note's ask (DR5) can check it: extruded
        letters fail an ask that said 2D, flat ones fail an ask that said 3D
      And the skill's core says: the scene being 3D decides nothing about a title, a lyric or a graphic

  # =====================================================================================================================
  Rule: Part 8. Review in motion and in sound (finding 7)

    @needs:DR10
    Scenario: DR20. What is reviewed is what plays
      Then `raea stage review` and `raea show` make, for each shot, a clip and a strip of every third frame; a
        single still is offered only for a thing that doesn't move
      And a still drawn alone must equal its frame in the continuous render: `raea render --at` stills are checked
        against the same frames stepped in sequence, and a difference is a finding naming the value that held
      And the measures of goal.feature Track G's catalogue run on any piece, as raea's named checks: eye trace and
        screen direction at a cut, an unmotivated jump in brightness or warmth, a subject lost from frame, a pop in a
        camera's path or lens, a hold too short to read
      And the project-local scripts the film grew (motion, framing, seat, light, frame jumps) are these checks,
        once, in raea, as systems over the world
      And a stage closes with every finding fixed or carried with a reason; a check that ran and wasn't read (its
        findings never queried by the session that ran it) is itself a finding

    @needs:DR20
    Scenario: DR21. Sound is measured against the picture, and what wasn't heard is said
      Given apps/check's sound rules (loudness, true peak, a bed against the voice, a sound that stops dead)
      Then the sound stage adds: each Cue's measured onset within a frame of its CausedBy; no Cue whose cause is no
        longer in the picture; nothing in the picture that should sound and has no Cue (from the blocking's
        impacts); the mix against the music by section
      And `raea stage review sound` posts the mix with its cues marked on a waveform under the strip
      And every post an agent makes ends with what it did not watch in motion and did not hear

  # =====================================================================================================================
  Rule: Part 9. Into the skill, proven, then used

    @needs:DR5,DR10,DR11,DR15,DR18,DR19,DR20
    Scenario: DR22. The raea skill teaches this
      Then the core of skills/skills/raea has: the stages in order; "restate the note, then its check, then the
        work"; "the medium is the person's word"; "never filter findings"; "a project's look is its own"; "every prop
        is a component"; and each stage's commands (the flags in the generated references)
      And craft/ROUTER.md's order of work and its "a change keeps; a redesign replaces" are rewritten on
        asks and checks
      And it lands as a raea-skills PR and a pin bump; the installed skill is that checkout

    @needs:DR22
    Scenario: DR23. An agent with only the skill does it right, and can't cheat it
      Given the skill evals (skills.feature SK7)
      Then new tasks, each scored by raea's findings, not by the agent's word:
        | task                                                                             | passes when                                       |
        | five notes on a piece, one holding three asks, one saying "2D"                    | seven asks restated; each check failed first; the medium checked |
        | a second round of notes on the same piece                                         | no regression of round one                        |
        | populate an office and a street from a taxonomy                                   | DR12's checks                                     |
        | a rider onto, on and off a seat                                                   | DR16's check, on every frame                      |
        | a lyric line behind a subject                                                     | DR19's check                                      |
        | a place asked for in one sentence                                                 | its plan written first; the floor met or fewer places offered |
        | a value that changes over a shot                                                  | reviewed as a clip; the still-against-sequence check clean |
      And every cheat of DR9 fails

    @needs:DR23
    Scenario: DR25. One short piece made this way, start to finish, before any long one
      Then a thirty-second piece goes through the stages in order with a person's notes over two rounds
      And its record shows: every note restated with its asks, each check failed first, no regression, no finding
        filtered, a look of its own, every prop a component over its floor, blocking closed before a camera, and
        every review a clip
      And only then is a longer film begun

  # =====================================================================================================================
  Rule: Order, and what is deliberately not here

    Scenario: DR26. The order to build in
      Then Part 1 (DR1 to DR4) first: the world, the one finding and checks as systems are what everything reads
      And Part 2 (DR5 to DR9) second: it changes how every later piece of this plan is itself reviewed
      And DR17 and DR20's still-against-sequence check early, as they are small and each would have stopped a
        failure of the film on its own
      And Parts 3 to 8 after, in their order; Part 9 as each part lands (its lines into the skill with it), the
        evals and the short piece last

    Scenario: DR27. Not in this plan
      Then no film is repaired here: the abandoned one stays as it is
      And no new kind of renderer, no new asset store, and no model or generation service is added
      And judging taste by a model is as it is (`raea judge`); here taste is the person's signature on a clip
