# Security policy

Rocci is experimental software. Treat it as a preview toolchain, not a
production security product.

## Reporting a vulnerability

Email **oss@rocci.dev** with a description of the issue, affected versions or
commits, and steps a maintainer can use to reproduce it. Do not open a public
GitHub issue for unfixed vulnerabilities.

If GitHub private vulnerability reporting is enabled on this repository, you
may use that instead of email.

There is no bug bounty. We will acknowledge reports we can act on and say
when a fix is published or why we are not treating the report as a
vulnerability.

A dedicated `security@rocci.dev` mailbox is not listed until it exists and
forwards.

## Scope

In scope: remote code execution, secret leakage in this repository's CI or
docs, and unsafe handling of untrusted `.rocci` / `.rocdown` input in the
compiler or preview hosts.

Out of scope for now: social-engineering of community channels, issues that
require a compromised local toolchain, and theoretical flaws in third-party
Roc or Datastar releases (report those upstream).
