# Rocci public-preview community launch kit

**Working draft — 17 August 2026**

This kit turns the branding recommendation into copy that can be reviewed and
posted after the repository launch gate is complete. It does not authorize a
public announcement, claim official Roc or Datastar status, or assume a
particular community channel remains appropriate.

## Before posting

Do not publish either message until all of these are true:

- the repository contains the promised license texts and community-health
  documents;
- a new user has completed the documented installation from a clean checkout;
- the tested Roc revision, Datastar pin, platforms, and known limitations are
  visible from the landing page or first README screen;
- the demo, repository, and feedback links are public and work in a signed-out
  browser;
- the feedback thread has a named maintainer and a response expectation;
- the Rocci Project is described as independent wherever Roc or Datastar is
  named.

Use a tagged preview revision in demos. Replace every bracketed placeholder
below, then have one person unfamiliar with the project read the final post
before publishing.

## Shared 30-second explanation

> Rocci is an independent open-source toolchain for writing HTML components
> and Markdown-first documents in Roc. It adds opt-in server-driven interaction
> with Datastar and can run the same HTTP interface in a desktop webview. The
> project is experimental and is not an official Roc or Datastar project.

## Roc community post

**Suggested subject:** Early preview: Rocci, web and desktop interface tools for Roc

Hi! I am preparing an early public preview of **Rocci**, an independent
open-source toolchain for building HTML interfaces and Markdown-first documents
in Roc.

Rocci currently has three author-facing parts:

- `.rocci` templates lower to ordinary Roc `Html`;
- Rocdown is a Markdown-first document format with explicit executable regions;
- Rocci Docs is the public name for the current `rocs` static documentation
  engine.

There is also an experimental Datastar path for server-driven interaction and a
desktop host that opens the same local HTTP app in a native webview.

This is a design preview, not a production recommendation, and Rocci is not an
official Roc project. I would value focused feedback before the name, hierarchy,
and visual identity harden.

- 90-second demo: [demo URL]
- repository and five-minute example: [repository URL]
- architecture and current limitations: [overview URL]
- structured feedback thread: [feedback URL]

The most useful questions for this round are:

1. How did you pronounce “Rocci” on first reading, and what did you expect it
   to do?
2. After one screen, is the relationship between Rocci, Rocdown, and Rocci Docs
   clear?
3. Does “web and desktop interface tools for Roc” describe the project without
   sounding official?
4. Which single workflow would make you most likely to try it?
5. For the early identity directions, does the folded visual language feel
   related to Roc, derivative of Roc, or unrelated?

Syntax and compiler-compatibility feedback are welcome too, but I will keep
those in separate threads so the questions remain answerable. I will publish a
short synthesis after two weeks, including what changed, what did not, and what
is still undecided.

## Datastar community post

**Suggested subject:** Feedback wanted: experimental Roc backend integration with Datastar

Hi! I am testing **Rocci**, an independent open-source Roc toolchain, with
Datastar for server-driven HTML interaction. Rocci is not an official Datastar
integration or SDK.

The focused example shows [one sentence describing the exact demo], using the
pinned Datastar version documented in the repository. I would especially value
review of the integration boundary rather than broad feedback on the template
syntax.

- short demo: [demo URL]
- example source and compatibility note: [source URL]
- focused feedback thread: [feedback URL]

Questions:

1. Are the event and patch semantics represented accurately?
2. Does the example follow current Datastar conventions?
3. Which compatibility or failure case should be documented before others try
   it?
4. Would a maintained Roc example be useful, and what would it need before an
   SDK discussion made sense?

I will report the resulting corrections and explicit non-decisions back in the
public feedback thread.

## Feedback issue template

**Title:** Public preview feedback: name, hierarchy, first workflow

Thank you for evaluating the Rocci public preview. Partial answers are useful.
Please avoid sharing private employer or project details.

- **Your context (optional):** Roc user / Datastar user / neither / other
- **First-read pronunciation of Rocci:**
- **What you expected the project to do:**
- **Hierarchy:** In your own words, how are Rocci, Rocdown, and Rocci Docs
  connected?
- **Official-status signal:** Did any wording or visual make the project seem
  official to Roc or Datastar? Which one?
- **First workflow:** What would you try first, and what would block you?
- **Identity direction:** Folded letter / non-letter modular symbol / wordmark
  only / no preference. Why?
- **May we quote this response in the public synthesis?** Yes with attribution /
  yes anonymously / no

Bug reports, security reports, and syntax proposals should use their dedicated
routes rather than this feedback issue.

## Response and moderation protocol

- Acknowledge new feedback within the published response window; do not promise
  a turnaround the maintainer cannot sustain.
- Separate observations, preferences, and reproducible failures in replies.
- Never treat reactions, silence, or a single prominent voice as consensus.
- Ask explicit permission before quoting or identifying a participant.
- Move security disclosures to the private security route immediately.
- Apply the published code of conduct consistently, including to maintainers.
- Close the round on the stated date and preserve late responses for the next
  synthesis rather than silently changing the sample.

## Two-week synthesis template

# Rocci public-preview feedback synthesis — [dates]

## Participation and limits

- Responses reviewed: [count]
- Roc community: [count]
- Datastar community: [count]
- Other or undisclosed: [count]
- Collection method and known sampling limits: [text]

## Findings by question

For each question record:

- the repeated themes and approximate counts;
- representative quotations only where permission exists;
- contradictory or minority evidence;
- what the evidence can and cannot support.

## Decision log

| Topic | Evidence | Decision | State | Revisit trigger |
| --- | --- | --- | --- | --- |
| Name and pronunciation | [summary] | [decision] | retained / changed / deferred / rejected | [condition] |
| Brand hierarchy | [summary] | [decision] | retained / changed / deferred / rejected | [condition] |
| Project descriptor | [summary] | [decision] | retained / changed / deferred / rejected | [condition] |
| First workflow | [summary] | [decision] | retained / changed / deferred / rejected | [condition] |
| Identity route | [summary] | [decision] | retained / changed / deferred / rejected | [condition] |

## Next narrow milestone

State one milestone, its acceptance checks, and two to five bounded contribution
opportunities. Link back to raw public feedback where consent and platform rules
allow it.

## Suggested publishing sequence

1. Soft-open the repository and feedback issue without a broad announcement.
2. Ask one or two Roc community regulars where the preview belongs now.
3. Post to Roc first and let installation or positioning problems surface.
4. Correct material issues before sharing the focused Datastar example.
5. Close and synthesize the round after two weeks; publish decisions before the
   next announcement.

