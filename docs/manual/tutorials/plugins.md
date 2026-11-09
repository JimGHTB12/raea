---
title: "Build and install plugins"
summary: "Install a plugin anyone made from npm, a Git link or a folder, read what it asks for before it runs, then build your own: a tool on feed items, a panel beside the feed, or a new kind of item, reloading as you save."
order: 11
sources: ["docs/manual/guides/plugins.md", "docs/ecosystem.feature", "apps/app/src/plugins/InstalledSection.tsx", "apps/app/src/plugins/manifest.ts", "apps/app/host/packages.ts", "apps/app/src/artifacts/bridge.ts"]
---

A plugin adds something to raea that it doesn't have yet. This tutorial installs one, reads what it asks for, uses it
in the feed, and then builds a small plugin of your own and shares it.

## What a plugin is

A plugin is an ordinary npm package with a raea section in its package file. It can add to the app: a tool on feed
items, a panel beside the feed, or a new kind of feed item for your agents to post. It can also add components that
your compositions import. You install a plugin once, and it's there in every project you open.

## Install one

Open Plugins in the sidebar. Under Installed, type what you want to install. That can be a package name from npm, a Git
link, or a folder on this computer. Then press Install. Installing runs none of the plugin's code.

## Read it, then approve it

The plugin arrives switched off, on a card that says three things: what it adds, what it may do, and which websites it
may reach. Read the card. If it looks right, press Approve and turn on. If it doesn't, cancel the install, and nothing
it contains has run.

## Use it in the feed

Open an item in the feed. The plugin's tool is a button beside the item, and pressing it opens the tool in a pane next
to it. A panel opens from its toggle on the feed itself. Each plugin runs in a sandbox of its own, so it can't read
your keys, reach the rest of the app, or touch another plugin. If it asks for something it wasn't given, the app
refuses and says which permission it lacked. If it tries to reach a website it didn't name, that is blocked too.

## Build your own

A plugin is a folder with a package file and a React file for each part it adds. The package file names each part,
where it shows, the permissions it needs, and the websites it may reach. The React file talks to the app through one
small library, at raea slash app. It can read the item it's on, leave a note, read the project's items, or show you a
short notice, each only if you allowed it.

## Try it as you save

Install your own plugin from its folder, the same way as before, and approve it. A folder plugin is linked, not
copied, so every time you save, the app rebuilds it and the open tool reloads in place. A mistake shows in the tool
with its file and line until you fix it. You can also ask your agent to write the plugin with you, or to post an item
that your plugin draws.

## Share it

When it works, share it the way you'd share any npm package. Publish it to npm, or push the folder to a Git
repository. Other people install it by its name or its link, and they read its card and approve it, just as you did.
To stop a plugin, turn it off, and it stops everywhere. Remove takes it away completely, and neither touches your
projects.

```json
{
  "name": "@you/raea-palette",
  "version": "0.1.0",
  "raea": {
    "title": "Palette",
    "app": {
      "contributes": [{ "point": "tool", "id": "palette", "title": "Palette", "entry": "./app/Palette.tsx" }],
      "permissions": ["comments:write", "notify"],
      "network": []
    }
  }
}
```

```tsx
// app/Palette.tsx: a tool on items, pinning a palette as a note
import { app, useItem } from "@raea/app"

export default function Palette() {
  const item = useItem()
  return (
    <button className="m-4 rounded-md border border-line px-3 py-1.5 text-xs text-ink" onClick={() => app.comment("Palette: #1d2a3a #c58b4e #efe4d2")}>
      Pin a palette to {item?.title}
    </button>
  )
}
```

```sh
npm publish                      # share it on npm: people install it by name
git push                         # or share the folder on Git: people install it by its link
```

The full reference, with every manifest key and every call a plugin can make, is the guide
[Build and share plugins](../guides/plugins.md).
