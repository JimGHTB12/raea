@plan
Feature: raea's plugin ecosystem: extending the engine and the app, made, shared and installed the same way
  Shane, 11-09: "write a plan for this, that fully charts out what a plugin ecosystem looks like both for the engine
  itself and the app". This is that plan: what exists, the one package format both sides share, the engine's three
  tiers, the app's extension host, and the order to build it in. Nothing here is built yet beyond what "What exists"
  names. The Bevy plugin catalog's own work stays in docs/plugins.feature.

  What exists (11-09):
    - The engine is extended through its SDK. TypeScript (components, parts, systems on BevyApp, npm packages through
      `raea add`, ready parts through `raea part add`) runs on the shipped raea with no build, and a play reloads the
      edit on save (runtime/live, docs/bevy-ts.feature "hot reload: partial").
    - Shader files are data: a Material's or a MaterialExtension's WGSL is compiled as written (docs/control.feature
      FC2a, FC2b), so a surface ships as a file.
    - Native Bevy plugins (Rust crates) are declared in a project's package.json (`raea.plugins`), built into
      raea-bevy-sdk behind cargo features, and typed in TypeScript from the built App's TypeRegistry
      (tools/bevy-plugins.sh, sdk/raea/docs/bevy-plugins.md). The release builds raea with the catalog's features
      (tools/release.sh), so the catalog's plugins work in the shipped binary; any other crate needs raea built from
      source. Rust can't be loaded into a running raea: Bevy has no stable ABI.
    - The app (raea feed, apps/app) is a React page drawn by Blitz in raea's Bevy window, its JS in QuickJS, with a
      Node host for everything privileged (/host/*). The shipped page is one bundle that loads no code of anyone
      else's. In development, blitz/dev.mjs already runs the page's modules through Vite's module runner and pushes
      hot updates into the window (raea:hmr): that is the loader an extension host needs.
    - The Plugins page is a fixed catalog of integrations (src/plugins/catalog.ts): ACP agents, accounts on networks,
      keyed generation services. Its entries are connections, not code.
    - The app's extension points today are data and agents: any ACP agent, the feed tools an agent calls
      (feed_post, controller_write, item_tags...), controllers as TypeScript files, and artifacts (a React document
      in a sandboxed frame with one result hook, src/detail/views/Artifact.tsx). The artifact entry is still a
      hard-coded demo (src/artifacts/main.tsx); an agent can't ship its own artifact code yet.

  Rule: The two answers this plan rests on

    Scenario: EC0a. The engine's extension surface is the SDK, plus native plugins under it
      Then extending the engine means writing against raea's SDK, as compositions already do: a package of components,
        systems, shaders and parts any composition imports
      And under the SDK sit native Bevy plugins, which are the one kind of engine extension that needs a build
      And so the engine side is mostly packaging what exists (a manifest, `raea add`, a registry, compatibility), not
        a new runtime

    Scenario: EC0b. The app is the side that needs a runtime: a loader, a sandbox, a bridge and hot reload
      Then the app has no way to run someone else's code today, and that is the work: load an extension's modules into
        the running app, keep them away from the person's keys and the host, give them a typed bridge, and reload
        them on save while they're being made
      And the reload already exists for development (blitz/dev.mjs's module runner and raea:hmr); the plan turns it
        into the extension host's dev mode and gives the shipped app a loader for built bundles

  Rule: One package format for the engine and the app

    Scenario: EC1. A raea package is an npm package with a "raea" key
      Given a package.json whose "raea" key says what the package adds:
        """
        "raea": {
          "raea": ">=0.1.0 <0.3",
          "engine": { "exports": "./src/index.ts", "shaders": "./shaders", "plugins": { "hanabi": { ... } } },
          "app": { "entry": "./app/index.tsx", "contributes": { "tools": [...], "itemTypes": [...], "panels": [...] } },
          "permissions": ["feed.read", "comments.write", "net:api.example.com"]
        }
        """
      Then `raea add NAME` installs it into a project (raea.lock, node_modules), and `raea add --global NAME` into
        ~/.raea for every project, as `raea add` does for npm packages now
      And a package can be engine only, app only, or both (a particle editor: the components and the panel that
        tunes them)
      And the "raea" range is checked at install and at load: a package made for another raea is refused with the
        version it wants and the one it found, never loaded and left to fail

    Scenario: EC2. Installed packages are visible and reversible
      Then the Plugins page lists installed packages beside the integrations, by project and global, each with what it
        adds, its permissions, its version and its source
      And each can be turned off per project without uninstalling, and removed, and either leaves no trace in the
        project's files beyond raea.lock

  Rule: The engine: three tiers, by how they load

    Scenario: EC3. TypeScript packages run on the shipped raea and reload in play
      Given a package of components, systems or parts written against raea's SDK
      Then any composition imports it by name, renders and plays it with the shipped raea, and a play reloads it when
        its files change, as an edit reloads today
      And `raea check` holds a package to the rules a composition is held to (values from the frame, render equals
        play), so an extension can't quietly break a render's determinism

    Scenario: EC4. Shader and asset packages are files, compiled when loaded
      Then a package's WGSL materials, extensions and post passes, and its models, textures, fonts and sounds, load from
        the package as a composition's own do, and reload in play when they change

    Scenario: EC5. A native plugin in the catalog works everywhere; one outside it builds a project's own raea
      Given a package that declares a native Bevy plugin
      Then a plugin in raea's catalog needs nothing more: the shipped raea has it, and the package carries only its
        TypeScript
      And a plugin outside the catalog is built by `raea build --plugins` (tools/bevy-plugins.sh build, made a verb),
        which makes this project's own raea binary with the plugin in it, caches it by the plugin lock's hash, and runs
        renders and plays on it
      And a plugin author who wants every raea to have it opens a PR to the catalog: it is reviewed, pinned per Bevy
        minor, proven by `run.sh catalog`, and ships in the next release
      And loading Rust into a running raea is out of scope while Bevy has no stable ABI; logic that must be native and
        loadable is a research question for a WASM tier (wasmtime, a raea host API), not a promise

  Rule: The app: an extension host

    Scenario: EC6. An app extension runs sandboxed and talks to the app only through a bridge
      Given a package's app entry
      Then the app runs each extension in a sandboxed frame of its own (as an artifact runs now: scripts, no same
        origin), never in the app's own JS realm, so it can't read keys, call /host/* or touch another extension
      And it talks to the app through `@raea/app`, a typed bridge over postMessage: read the project, the feed and an
        item; read and write comments and pins; post items; read and write controllers; call raea's CLI verbs the
        manifest's permissions allow; open views; show a notice
      And every bridge call is checked against the permissions the person approved, and refused with which permission
        it lacked

    Scenario: EC7. The places an extension can add to
      Then an extension contributes from its manifest, each a component the app mounts in that extension's frame:
        | point          | what it is                                                                              |
        | detail tool    | a button in a detail view's toolbar, with the frame, the pins and the selection        |
        | rail tab       | a tab beside Notes, Layers and Controls on an item                                     |
        | item type      | a new kind of feed item: its card and its detail view, posted by agents with feed_post |
        | panel          | a panel in the feed or the sidebar, as the Agents panel is                             |
        | command        | a named action with an optional key, run from the command menu                         |
        | agent tool     | a tool every agent in the project gets over ACP, answered by the extension              |
        | settings page  | its own settings under Settings, stored per project or per person                      |
      And an artifact becomes the simplest extension: an agent's own React document, bundled and run in its frame, its
        result hook one bridge call, so the hard-coded demo goes

    Scenario: EC8. Making an extension reloads it in the running app
      Given a person or an agent working on an extension
      When they run `raea app dev DIR` (the extension's folder) with the app open
      Then the app's host serves the extension's modules through Vite's module runner, as blitz/dev.mjs serves the feed
        now, and pushes each saved change into the extension's frame (raea:hmr), keeping its state where React can
      And a compile error shows in the extension's place with its file and line, as the brief's compiler does
      And `raea app check DIR` mounts each contribution headless with recorded app state and steps, writes a PNG per
        step and a tree.json, as `raea app check` does for an app now
      And the shipped app loads only built bundles (`raea app bundle DIR`): no dev server, no network fetch of code

    Scenario: EC9. Agents make extensions the person approves
      Given a person asking their agent for a tool the app doesn't have
      Then the agent writes the extension in the project, checks it with `raea app check DIR`, and proposes it
      And it runs only after the person reads what it adds and what it may do, and presses Install
      And the raea skill teaches it (a guide beside motion/interface: an extension's manifest, the bridge, the checks)

  Rule: Sharing: one registry, and trust that's earned

    Scenario: EC10. npm is the store; raea-api is the index
      Then packages are published to npm (keyword raea-package), so authors use the tools they know and raea hosts no
        code
      And raea-api keeps an index of them, read by the Plugins page's browse view and by `raea search-packages`: name,
        what it adds, permissions, the raea range, downloads, and whether its build is verified
      And a package is installed by its npm name and version, never from a URL

    Scenario: EC11. Verified packages show where they came from
      Then a package published with npm provenance (built from a public repo by CI) shows its source and commit
      And raea-reviewed packages are marked verified; others are marked community and say so at install
      And a package can be pulled from the index, which turns it off where it is installed, with the reason shown

  Rule: The order to build it in

    Scenario: EC12. Phases
      Then phase 1 (after v0.1.0): the manifest and `raea add` for engine packages (EC1, EC3, EC4), the extension host
        with the bridge and one point, the detail tool (EC6), `raea app dev` with hot reload (EC8), and artifacts as
        extensions (EC7's last line)
      And phase 2: the other points (item types, rail tabs, panels, commands, agent tools, settings), permissions on
        the Plugins page (EC2), agents making extensions (EC9)
      And phase 3: the registry and index (EC10, EC11), `raea build --plugins` as a verb and the catalog's PR process
        (EC5)
      And phase 4: research only, a WASM tier for native logic loaded at run time

  Rule: Open questions for Shane

    Scenario: EC13. Decisions this plan needs
      Then may an extension reach the network, and if so only to the domains its manifest names?
      And may an extension restyle the app itself (a theme), or only draw inside its own frames?
      And are packages installed per project by default (they travel with the repo) or per person?
      And will packages ever be sold through raea, or stay free with raea taking no part?
