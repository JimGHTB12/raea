---
title: "Connect your agent"
summary: "Bring the AI you already pay for over ACP, add agents to your workspace with their own instructions, and start work with the Director."
order: 2
sources: ["apps/app/src/plugins", "apps/app/src/settings/Agents.tsx", "apps/app/src/agent/AgentEditor.tsx", "apps/app/src/acp", "apps/app/src/chat"]
tutorial: "connect-your-agent"
---

raea runs the agent you already use. This tutorial connects one, shows where your workspace's agents live, and
introduces the Director, the agent you talk to first.

## How raea uses your agent

raea doesn't sell AI. It runs the agent you already pay for, on your own computer, through the Agent Client Protocol,
called ACP for short. ACP is an open standard for talking to coding agents. raea starts your agent, hands it the
project's brief and raea's own tools, and shows its work in the feed. Your agent signs in with its own account, and raea
never sees or keeps that login.

## Pick your agent in Plugins

Open Plugins in the sidebar. The Agents section lists the agents raea knows: Claude, ChatGPT through Codex, Kimi,
Gemini, and OpenCode, which reaches any model on OpenRouter. Pick the one you pay for and press Connect. raea installs
it if this computer doesn't have it yet, and opens its sign-in in your browser. Once it's signed in, raea sends it one
short prompt, to check that it answers.

## Bring your own

If your agent isn't listed, choose Your own agent, and give raea the command that starts it. Any agent that speaks ACP
over standard input and output will work. raea starts it in the project's folder, so it reads and writes the project's
files directly.

## Agents in your workspace

Now open Settings, then Agents. Each agent here is a role, such as the Director, a Designer or an Animator, and each one
says which AI runs it and whether it's signed in. Press New agent to add another. Give it a name, choose what runs it,
and write its instructions in plain words. Agents belong to the workspace, so every project can call on them.

## The Director

Every workspace has a Director. It's the agent you talk to in New chat. It reads the whole workspace, makes projects,
and builds controllers when you ask for them. Write what you want in New chat, pick a project under the box, and the
Director starts the work there.

## Agents in a terminal

You can also work from your coding agent's own terminal. Install raea's skill with raea skills install, naming your
agent. The skill teaches it raea's loop: read the project, write the composition, check it, look at stills, then render.
The agents raea starts get the same skill, plus the feed's own tools.

```sh
raea skills install --agent codex     # or kimi, gemini; Claude Code gets it with raea itself
raea skill                            # print the skill your installed raea ships
```

Next: [A tour of the app](tour.md).
