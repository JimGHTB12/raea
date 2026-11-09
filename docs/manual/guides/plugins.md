---
title: "Build and share plugins"
summary: "A plugin is an npm package with a raea key: components your compositions import, and tools, panels and item types in the raea app, each sandboxed with only the permissions and hosts the person approved. Build one in a folder, test it with hot reload, and share it on npm or Git."
order: 5
sources: ["docs/ecosystem.feature", "apps/app/src/plugins/manifest.ts", "apps/app/host/packages.ts", "apps/app/src/artifacts/bridge.ts", "apps/app/src/artifacts/protocol.ts", "engine/bundle/src/edit.rs"]
---

A plugin adds to raea what it doesn't have yet. It's an ordinary npm package whose package.json has a `"raea"` key,
and it can add to either side of raea, or to both:

- **The engine.** Components, parts and systems your compositions import by the package's name, like any other
  package. raea looks for them in your plugins after the project's own packages, so a component installed once works
  in every project.
- **The app.** A tool on feed items, a panel beside the feed, or a new kind of feed item that agents post. Each one is a
  React `.tsx` file that the app runs in a sandboxed frame of its own.

Plugins are installed per person. The app keeps them in `~/.raea/plugins` (or `RAEA_PLUGINS`), and they're there in
every project you open.

## The manifest

```json
{
  "name": "@you/raea-palette",
  "version": "0.1.0",
  "description": "Pull a palette from any still and pin it to the item.",
  "raea": {
    "title": "Palette",
    "raea": ">=0.1.0",
    "engine": { "imports": ["@you/raea-palette"] },
    "app": {
      "contributes": [
        { "point": "tool", "id": "palette", "title": "Palette", "entry": "./app/Palette.tsx", "on": ["look", "watch"] },
        { "point": "panel", "id": "swatches", "title": "Swatches", "entry": "./app/Swatches.tsx" },
        { "point": "item", "id": "board", "title": "Palette board", "entry": "./app/Board.tsx" }
      ],
      "permissions": ["items:read", "comments:write", "notify"],
      "network": ["api.example.com"]
    }
  }
}
```

| key | what it says |
|---|---|
| `title` | the name the person sees on the Plugins page |
| `raea` | the raea versions it works with, as an npm range |
| `engine.imports` | what compositions import from it (the package's own `exports` are what actually resolve) |
| `app.contributes` | each part it adds to the app: `point` (`tool`, `panel` or `item`), an `id` of lowercase letters, digits and dashes, a `title`, and an `entry`, a `.tsx` inside the package |
| `on` | for a tool: the item types it shows on (`watch`, `look`, `listen`, `play`, `artifact`, `plugin`); every type when left out |
| `app.permissions` | what it may ask of the app beyond reading what it's shown on |
| `app.network` | the hosts its code may reach, exactly (`api.example.com`) or a subdomain wildcard (`*.example.com`); none when empty |

A package without a `"raea"` key isn't a plugin, and the app refuses it at install, naming every problem it found.

## Where a contribution shows

- **tool**: a button in an item's detail view. Pressing it opens your component in a pane beside the item, and it
  gets the item it's on.
- **panel**: a toggle on the feed that opens your component beside it, for the project as a whole.
- **item**: a kind of feed item. An agent posts one with `feed_post`, type `plugin`, naming your plugin and the view,
  as `"@you/raea-palette#board"`, and a JSON file that becomes the item's data. The feed shows the item with your
  component. If someone opens the item without your plugin, it says which plugin it needs.

## Talking to the app: @raea/app

Your component is the `.tsx` file's default export. It imports `react`, `@raea/app` and its own files (`.ts`, `.tsx`,
`.js`, `.json`), plus any packages it depends on. Tailwind classes work and use the app's own colours, so `text-ink` and
`bg-raised` look right in both themes.

```tsx
import { useState } from "react"
import { app, usePlugin, useItem } from "@raea/app"

export default function Palette() {
  const ctx = usePlugin()   // where it's shown: point, project, itemId, and an item's data
  const item = useItem()    // the item it's on: title, the agent's ask, its media with URLs (null in a panel)
  const [said, setSaid] = useState("")
  const pin = async () => {
    await app.comment("Palette: #1d2a3a #c58b4e #efe4d2")
    await app.notify("Pinned the palette")
    setSaid("pinned")
  }
  return (
    <main className="p-4 text-ink">
      <h1 className="text-sm font-medium">{item?.title}</h1>
      <button className="mt-3 rounded-md border border-line px-3 py-1.5 text-xs" onClick={pin}>Pin the palette</button>
      <p className="mt-2 text-xs text-ink-3">{said}</p>
    </main>
  )
}
```

| call | needs | what it does |
|---|---|---|
| `app.context()` / `usePlugin()` | nothing | the plugin, the contribution, its point, the project, the item it's on, a plugin item's data |
| `app.item()` / `useItem()` | nothing | the item it's shown on, when there is one |
| `app.items()` | `items:read` | the project's feed items and their notes |
| `app.comment(text, pin?, itemId?)` | `comments:write` | a note on the item, optionally pinned to a place on it (0 to 1 across and down), or on another item by id |
| `app.notify(text)` | `notify` | a short notice for the person |

A call without its permission is refused with an error that names the missing permission, for example "items needs the
items:read permission, which this plugin wasn't given". A call that doesn't exist is refused and the error lists the
calls that do.

## What a plugin can't do

The frame is a sandbox. It has no access to the app's own code, your keys, the app's host or another plugin's
frame. `@raea/app` is its only way to reach the app, and only `fetch`, `XMLHttpRequest`, `WebSocket`,
`EventSource` and `sendBeacon` calls to the hosts in `network` get through. Anything else fails with an error that
names the hosts the plugin may reach. Imports of Node's built-in modules (`node:fs` and the like) are refused, and so
are files outside the plugin's folder.

## Build and test it

1. Make a folder with the package.json above and your `.tsx` files.
2. In the raea app, open **Plugins**. Under **Installed**, type the folder's path (`~/code/raea-palette`) and press
   **Install**.
3. The plugin installs **off**. Its card shows what it adds, what it may do and which hosts it reaches. Press
   **Approve and turn on**.
4. Open an item and press your tool, or toggle your panel on the feed.
5. Edit the `.tsx` and save. A folder plugin is linked, not copied, so the app rebuilds it and the frame reloads in
   place. A compile error shows in the frame with its file and line.

To try an item view, ask your agent to post one: "post a palette board for this still with
`@you/raea-palette#board`". It writes the JSON and posts it with `feed_post`.

## Components for compositions

The same package can export components for the engine. Once the plugin is installed, any composition imports them by
the package's name:

```tsx
import { Grain } from "@you/raea-grain"
```

raea resolves an import from the project's own `node_modules` first, then from your plugins. A project that installs
its own copy of a package keeps that copy. Components follow the rules every composition follows: every value comes
from the frame, so a render matches a play. The [SDK](../reference/sdk/index.md) lists what they can import.

## Share it

A plugin is shared the way any npm package is. Nobody uploads anything to raea.

- **npm**: `npm publish`. People install it by typing its name (`@you/raea-palette`) on the Plugins page.
- **Git**: push the folder to a repository. People install it with the Git URL
  (`github:you/raea-palette` or `git+https://github.com/you/raea-palette.git`).

Installing runs none of the package's code. npm's install scripts are off, and nothing in the plugin runs until the
person has read its card and approved it. Turning a plugin off stops it everywhere without uninstalling it, and
**Remove** uninstalls it. Neither touches a project's files.

## Not built yet

- `raea add` and `raea app dev` verbs for plugins.
- Checking a plugin's `raea` range when it loads.
- Rail tabs, commands, agent tools and settings pages as contribution points.
- A browsable index of plugins with verified sources.

A native Bevy plugin (a Rust crate) still has to be in raea's catalog or built into raea from source. The Bevy plugin
guide in the raea skill covers those.
