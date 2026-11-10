@spec
Feature: Agents in the raea app: chats run on a provider, agents are optional templates, and whoever posts an item hears back
  Shane, 2026-11-09: "i want to make sure if you upload a video you are the subscribing agent to recieve the hook bakc
  to the session. and lets delte all the agent types we made default or have on the system. that said what we could
  ahve is a way for an agent to self assign its role if intiialized outsid eof raea. and if assigned in raea it would
  be set automatically but i need a way to start a new chat with just athe provider i dont have to use an agent its
  just a template type of skills and prompts d things"

  What the app is today (apps/app, read 11-09):
    - Agents are workspace members (src/model/agents.ts). The fixtures ship eight (Boards, Grade, Score, Checker,
      Director, Builder, Playtester, Cutter), and the page makes a Director and a Critic in every workspace the first
      time it needs them (builtIn). New chat starts a session with the workspace's Director (src/chat/NewChat.tsx); the
      feed's composer files a card for @director.
    - An item posted by an agent in raea carries its session (item.connection). A person's note goes to that session
      as its next turn if the session is open in the page (src/agent/notes.ts promptSession), else only into its log.
      Comments on the picture wait for "Send N edits to <agent>" (docs/review.feature R1e).
    - A session outside raea (a Claude Code session, a script) posts by writing <project>/.raea/feed/<id>.json. It has
      no session raea can reach: the launch film's 25 comments (i-acp-mw9k336z-2) sat in its JSON until Shane told
      the session to read them. That item borrowed a fixture's connection (c-plan), so the app named the Director as
      its author.

  The model this file sets:
    1. A chat runs on a provider: an ACP agent from the registry (Kimi, Claude, Codex, Gemini, ...) or a command of
       the person's own. That is all a chat needs.
    2. An agent is an optional template on top of a provider: a name and role, instructions, skills, tools. The app
       and raea ship none; a person makes them.
    3. Whoever posts an item is its subscriber: what the person sends on it reaches that session, inside raea or out.
    4. A session states its own role. One started from a template in raea gets the template's role without asking;
       one started outside raea says who it is when it posts, and may change it.

  Background:
    Given the app is apps/app (raea-feed): src/chat, src/agent, src/model/agents.ts, host/acp, host/feed-items.ts
    And the starter projects (RaeaStarters) and the e2e fixtures are sample content, not the person's agents

  Rule: A1. A new chat needs only a provider
    # Status 11-09: built (raea-feed 601d4ae). New chat's "Runs on" menu: PROVIDERS from the ACP plugins, Kimi first,
    # each marked as ai-find answers when the menu opens, then the workspace's templates; the pick is settings.chatOn.
    # A provider is an Agent with `provider` (model/agents.ts providerAgent, id p-<registry>-<workspace>), never in
    # agentsIn; every New chat session gets raea's context and tools (src/chat/useChat.ts). e2e "New chat runs on a
    # provider with no agent".
    Scenario: A1a. Starting a chat with just a provider
      When I open New chat
      Then I choose what runs it: a provider (Kimi first, then the registry's agents signed in on this Mac) or one of
        the workspace's agent templates, if there are any
      When I pick a provider and write
      Then a session starts on that provider in the project picked under the box, with raea's context and no
        template's instructions, and its page names the provider

    Scenario: A1b. A template is a choice, never a requirement
      Given a workspace with no agents
      Then New chat, the feed's composer and a project's chat all work, each on the workspace's default provider

  Rule: A2. raea ships no agent types
    # Status 11-09: built (raea-feed 601d4ae). No Director or Critic is made; the planner and critique post as raea
    # (raeaAgent, provider "raea"), tasks for it say @raea and an old brief's @director reads as raea. The built app's
    # sample world keeps its agents as `by` authors only (model/data.ts sampleAuthors); the dev page keeps them as
    # templates for the e2e suite. The host's model prompts (host/director.ts, compose.ts) still cast the planner as a
    # film director: that's the craft role, not an agent. e2e "raea ships no agents".
    Scenario: A2a. A new workspace has no agents
      Given a new install, or a workspace made in the app
      Then Settings → Agents and the sidebar's Agents list nothing until the person makes one, and no Director or
        Critic is made behind their back
    Scenario: A2b. What the built-ins did still happens
      Then the feed's composer starts a chat on the default provider in its project (and files the card), and the
        critique of a render is posted by raea itself, not by an agent named Critic
    Scenario: A2c. Sample content keeps its authors as names
      Given a starter project's items, made by the fixtures' agents
      Then they show their author's name and colour as written in the item, with no agent behind it

  Rule: A3. Whoever posts an item hears what's sent on it
    # Status 11-09: A3b built (raea-feed aee0ca2: Item.by, host/inbox.ts, POST /host/inbox; e2e "an outside poster's
    # item is theirs"). One line per send: {item, project, title, kind: note|edits, text, pin?, comments?: [{id, text,
    # kind, pin, at}], when}. A3a's "opened to take them" and A3c are next.

    Scenario: A3a. A session in raea
      Given an item posted with feed_post from a session in raea
      When I send notes or edits on it
      Then they reach that session as its next turn; a session that isn't open is opened to take them

    Scenario: A3b. A session outside raea subscribes when it posts
      Given a session outside raea posts an item, with `by` in its JSON: {"name", "role"?, "inbox"}
      Then the item shows `by.name` as its author, and the bar's send says "Send N edits to <name>"
      When I send them
      Then each is appended as one JSON line ({item, project, comment, text, pin, part?, kind, when}) to
        ~/.raea/inbox/<inbox>.jsonl, and marked sent on the item
      And the session hears it by following that file (`tail -n0 -F`, or its harness's monitor): that is its hook

    Scenario: A3c. An item no one subscribed to says so
      Given an item with neither a session nor `by.inbox`
      Then the bar says the edits have no one to go to, instead of a send that goes nowhere

  Rule: A4. A session states its role
    Scenario: A4a. Started from a template in raea
      Then the session's role is the template's, set when it opens, without asking
    Scenario: A4b. Started outside raea
      Then it states its role in `by.role` when it posts, and a later post may change it; the app shows the newest

  Rule: A5. What a session shows the person comes up in the raea app, as something they can use
    # Shane 11-09: "my agent working raea keeps making sheets in preview and not in raea how do we make it so when using
    # raea and raea app is installed new edits new sheets or examples or questions are pulled up in the raea app and
    # not preview. we should see them as interactive too. the whole point of raea is interactive artifacts and things
    # and recording them after making an answer."
    # Status 11-09: built. App side raea-feed 077e77f (host/show.ts, host-file.ts, raea:// links in src/App.tsx,
    # answers and results to the inbox); CLI engine/cli/src/show.ts; the skill's section 5 says it (raea-skills).
    Scenario: A5a. raea show from any session
      Given a session outside raea (a terminal agent) with the raea app installed
      When it runs `raea show FILE...` (stills, a video, a sound, a NAME.script.ts or .md, a .tsx) or
        `raea show --question "Q" --choice A --choice B`
      Then the app's host makes the item those files are (several stills a storyboard, a .tsx an artifact, a project's
        composition its editor, a question an ask) in the project named or the one the session's folder is in, with
        the session as its poster (`by`), and the window comes up on it (raea://item/<project>/<id>)
      And when the app isn't running it's started first; when it isn't installed the files open in the Mac's own viewer
        and the command says nothing was posted

    Scenario: A5b. The person's answer is recorded and reaches the session
      When the person leaves notes or edits, picks an answer, or an artifact sends its result
      Then it's kept on the item and appended to the session's inbox (A3b), which `raea show` names when it shows

    Scenario: A5c. Agents show, they don't open
      Then the raea skill tells every agent to show work with `raea show` (or feed_post inside the app), never to open
        a render, a sheet or a question in Preview or a browser while the app is installed
