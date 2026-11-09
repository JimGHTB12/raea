---
title: "Get started with raea"
summary: "Download the raea app, sign in on raea.io, link your Mac, and land in your first project with your agent already at work."
order: 1
sources: ["apps/site/src/account/onboarding.tsx", "apps/app/src/onboarding", "docs/onboarding.feature"]
tutorial: "get-started"
---

This tutorial takes you from nothing to your first piece of work. You need a Mac and an AI you already use, such as
Claude, ChatGPT, Kimi or Gemini. You don't need a terminal.

## What raea is

raea is a studio for making video, motion graphics, 3D and games with AI agents. You bring the agent you already pay
for. Your agents write each piece as code, raea renders it on your own computer, and everything they make arrives in
one feed, where you review it.

## Download the app

Start at raea.io, in the docs. Press Download for Mac, and open the raea app once it has installed. The first time it
opens, the app asks you to continue on raea.io. Press the button, and your browser opens with this computer's code
already filled in.

## Sign in and answer four questions

Sign in with GitHub, Google or Apple. Then raea asks four short questions. About you: what you make, and who it's for.
Your AI: the agent you already use. Your plan: Free, Pro or Team. And Your Mac: the computer you're linking. Each answer
shapes your first project, so it's worth a moment.

## Link your Mac

On the last step, press Link and finish. The app on your Mac signs in by itself, connects the agent you picked, and
makes your first project from your answers. You don't copy a code, and you never open a terminal.

## Your first project

The app opens on your work. Your agent is already making its first piece, and the project's Agents panel says so.
When it's done, the piece lands at the top of the feed. Open it, watch it, and leave a note on anything you'd change. The
agent answers with the next version. That's the whole loop, and the rest of these tutorials take each part of it in
turn.

```sh
# Prefer the terminal? The same raea, without the app:
curl -fsSL https://raea.io/install | sh
raea --version
```

Next: [Connect your agent](connect-your-agent.md).
