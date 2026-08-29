---
type: Research Report
title: Cursor My Machines worker on a personal Mac mini
description: "How to run Cursor Cloud Agent tool calls on a personal Mac mini via My Machines so tasks started from the iPhone app (or Agents Window) execute on that machine."
tags: [domain/rocci, concern/tooling, concern/ci, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-29T18:01:48Z }
stale_after: 2026-11-29
authority: exploratory
owners: [human:nils]
sources:
  - id: my-machines-docs
    resource: https://cursor.com/docs/cloud-agent/self-hosted-guides/my-machines
    title: Cursor My Machines documentation
    author: process:cursor-docs
    last_modified: 2026-08-29
  - id: mobile-docs
    resource: https://cursor.com/docs/cloud-agent/mobile
    title: Cursor for iOS documentation
    author: process:cursor-docs
    last_modified: 2026-08-29
  - id: pool-docs
    resource: https://cursor.com/docs/cloud-agent/self-hosted-guides/pool
    title: Cursor Self-Hosted Pool documentation
    author: process:cursor-docs
    last_modified: 2026-08-29
---

# Cursor My Machines worker on a personal Mac mini

## Purpose

Use an always-on Mac mini as the execution host for Cursor Cloud Agents started
from the iPhone app (or desktop Agents Window / cursor.com/agents). The agent
loop (planning and model inference) stays in Cursor's cloud; terminal commands,
file edits, browser actions, and stdio MCP servers run on the Mac mini.[^my-machines-docs]

This is **My Machines** (personal worker), not an Enterprise **Self-Hosted
Pool**. Pools need a service account API key and org admin settings; personal
workers use browser login or a personal user API key.[^my-machines-docs][^pool-docs]

## What runs where

| Concern | Where |
| --- | --- |
| Agent loop / models | Cursor cloud |
| Tool calls (shell, edits, local MCP stdio) | Mac mini worker |
| Session UI (phone, web, desktop Agents Window) | Cursor clients against the same account |
| Inbound ports / public IP / VPN | Not required; worker dials out over HTTPS |

Outbound HTTPS needed: `api2.cursor.sh`, `api2direct.cursor.sh`, and
`cloud-agent-artifacts.s3.us-east-1.amazonaws.com` for artifacts.[^my-machines-docs]

## Setup on the Mac mini

### 1. Install the Cursor agent CLI

```bash
curl https://cursor.com/install -fsS | bash
agent --version
```

### 2. Sign in with the same account as the phone

```bash
agent login
```

Use the Cursor account already used in the iOS app. My Machines workers are
tied to that user.[^my-machines-docs][^mobile-docs]

### 3. Start a named worker in the target repo

```bash
cd /path/to/rocci   # checkout whose git remote matches the repo you will target
agent worker start --name "mac-mini"
```

Keep the process running. By default the worker is long-lived and reusable
across sessions. The registered repository comes from that directory's git
remote; start a separate worker per repo checkout when serving more than one
repo.[^my-machines-docs]

For headless / always-on use, prefer a personal user API key from Cursor
Dashboard → API Keys:

```bash
agent worker start --name "mac-mini" --api-key "your-user-api-key"
```

Wrap that command in `launchd` (or similar) so it survives logout and reboot.
Do not use a service account key here; those only start pool workers
(`--pool`).[^my-machines-docs]

### 4. Run a task from the iPhone app

1. Open Cursor for iOS (same account).
2. Start or open a Cloud Agent.
3. In the **Run on** / environment picker, choose **mac-mini** under **My Machines** (sometimes labeled Remote Control).
4. Send the task.

Agents started on mobile also appear in the desktop Agents Window and at
cursor.com/agents for the same account.[^mobile-docs]

## Troubleshooting

```bash
cd /path/to/rocci
agent worker debug
# or
agent worker start --name "mac-mini" --debug
```

Confirm the worker process is still up, both devices use the same Cursor
account, and the checkout remote matches the repo named in the task.[^my-machines-docs]

## Out of scope for this note

- Enterprise Self-Hosted Pool fleets, Helm/operator, or Cloud Run worker pools.[^pool-docs]
- Cursor-managed cloud VM environments (`.cursor/environment.json` / dashboard snapshots). Those are a different base than My Machines.
- Automations targeting a personal My Machines worker (Enterprise pool territory per current product docs / forum clarifications).

[^my-machines-docs]: Official My Machines quickstart, auth rules, networking, and `worker=` targeting.
[^mobile-docs]: iOS app uses the same Cloud Agent backend; mobile-started agents show on desktop and web.
[^pool-docs]: Self-Hosted Pool is Enterprise-oriented shared infrastructure with service-account auth.
