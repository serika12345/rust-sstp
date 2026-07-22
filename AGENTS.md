# AGENTS.md

This file contains standing instructions for humans and agents changing this repository.

## 1. Purpose and scope

- Build a fast, safe, cross-platform SSTP client in Rust.
- Target Linux, Android, macOS, and iOS.
- Add the Flutter user interface only after the communication core is stable.
- Keep the initial core independent of user-interface and foreign-function-boundary concerns.
- Describe project scope through target systems and planned deliverables.
- Read `DESIGN_PLAN.md`, `docs/scope.md`, and `docs/architecture.md` for details.

Reason: client correctness, performance, and operating-system integration must remain independently testable.

Rejected alternative: starting with the user interface before the communication core is stable would make failure causes and completion criteria unclear.

## 2. Communication, language, and paths

- Lead with the result and the current PR.
- Separate verified facts from assumptions and cite the specification basis for decisions.
- State blockers, exclusions, and remaining work explicitly.
- Every material design decision in Japanese design documents must include `理由` and `不採用理由`.
- Write user-facing communication and repository natural language in Japanese while retaining established terms commonly used in Japanese software development, such as `domain`, `I/O`, and `PR`.
- `AGENTS.md` is the only repository document that may be written in English.
- Preserve exact spellings for proper nouns, source identifiers, language keywords, commands, file names, official names, and established technical or project terms.
- Do not use unexplained English jargon in Japanese text, but do not invent Japanese replacements for established terms commonly understood in Japan.
- When referring to a repository file or directory, write only a repository-relative path.
- Never write absolute paths, home-directory shorthand, file URIs, or relative paths that resolve outside the repository in documentation, comments, work records, or user-facing reports.
- Write command examples as if they are run from the repository root.

Reason: Japanese readers must be able to understand decisions without unexplained jargon, and repository references must work in every checkout without exposing a local directory layout.

Rejected alternative: mixed-language jargon and environment-specific paths shift interpretation and path resolution onto the reader and fail in other development or continuous-integration environments.

## 3. Before starting work

1. Read the active PR and completion criteria in `TODO.md`.
2. Read `docs/design-todo.md` and the latest entry in `docs/progress.md`.
3. Search the affected modules and tests before editing.
4. Enter the Nix environment through direnv or `nix develop`.
5. Start new behavior with a failing test.

Reason: changes must remain linear, reviewable, and independently integrable.

Rejected alternative: starting with implementation can accidentally decide unresolved questions or absorb responsibilities from another PR.

## 4. Canonical development environment

- `flake.nix` and `flake.lock` define the canonical toolchain.
- Keep `.envrc` limited to `use flake`; do not add machine-specific settings.
- Run Cargo commands inside the Nix environment.
- After changing dependency lock files, run `./scripts/verify-supply-chain`.

Reason: developers, continuous integration, and package builds must use the same inputs.

Rejected alternative: global tools or editor-bundled toolchains create differences that cannot be reproduced reliably.

## 5. Planning and progress

- `TODO.md` is the sole authority for implementation order, PRs, and completion criteria.
- `docs/design-todo.md` contains only unresolved design decisions.
- `docs/progress.md` contains execution results and verification evidence.
- Keep only one PR in progress.
- Record discoveries outside the current PR instead of implementing them immediately.

Reason: planned work, unresolved decisions, and verified results must retain distinct meanings.

Rejected alternative: combining everything in one task list makes completion criteria compete with research notes.

## 6. Branches and PRs

- Keep each change a small vertical slice.
- State completion criteria, included work, excluded work, verification, and stop conditions before editing.
- Update the design documents and PR plan before changing an established boundary.
- Do not mix unrelated formatting, renaming, or dependency updates into a change.

Reason: small coherent changes are easier to review, bisect, and revert.

Rejected alternative: broad cross-cutting changes make individual decisions and their evidence difficult to trace.

## 7. Architecture

- Preserve the dependency direction `sstp-harness -> sstp-session -> sstp-protocol`.
- Keep `sstp-protocol` and `sstp-session` free of I/O, operating-system dependencies, and `unsafe` code.
- State machines return typed actions instead of performing side effects.
- Inject operating-system facilities, TLS, sockets, clocks, randomness, and credentials at explicit boundaries.
- Do not create generic dumping grounds named `utils`, `common`, `helpers`, or `types`.
- Do not introduce a generic trait before at least two concrete consumers demonstrate the requirement.

Reason: each performance bottleneck must be measurable by layer, and operating-system adapters must remain thin.

Rejected alternative: a large generic core or speculative abstractions hide responsibilities, ownership, cancellation, and flow control.

## 8. Types and functions

- Prefer domain types in public interfaces.
- Do not expose raw bytes, integers, or strings while leaving their wire meaning to callers.
- Validate invariants in constructors so constructed values remain valid.
- Functions without I/O side effects return meaningful values instead of hiding results in `()`.
- Errors use enumerable categories and preserve their source when wrapping another error.
- Public names use protocol or responsibility vocabulary.

Reason: type checking should prevent invalid states, unit mistakes, and wire-format confusion.

Rejected alternative: primitive values plus comments cannot prevent callers from mixing meanings.

## 9. Unsafe code and secrets

- Keep `#![forbid(unsafe_code)]` in pure crates.
- Restrict `unsafe` to future operating-system or foreign-function boundaries.
- Document the invariant for every `unsafe` block with a `SAFETY:` comment.
- Never emit passwords, certificates, keys, or packet bodies in normal logs.
- Avoid unnecessary secret cloning and keep secret lifetimes narrow.

Reason: memory-safety review and secret exposure must be limited to the smallest possible surface.

Rejected alternative: repository-wide `unsafe` permission or unrestricted diagnostic dumps make meaningful VPN-boundary auditing impossible.

## 10. Performance

- Measure before optimizing.
- Measure throughput, latency, allocations, copies, system calls, and wake-ups by layer.
- Do not place PTY, TTY, or an external PPP process in the shared data path.
- Bound every queue and make flow control and cancellation explicit.
- Prefer ownership transfer over unconditional buffer cloning.

Reason: user-space relays and process-boundary copies are known sources of avoidable overhead, especially on macOS.

Rejected alternative: a runtime or language being generally fast does not prove that the actual packet path is fast.

## 11. Tests

- Build coverage in this order: domain unit tests, codec vectors, state-machine property tests, adapter contracts, interoperability tests, and benchmarks.
- Do not put real time, uncontrolled randomness, or direct input/output into pure tests.
- Add a reproducing test before fixing a bug.
- Keep external-server tests separate from required unit tests and enable them explicitly.

Reason: fast mandatory verification and environment-dependent interoperability testing serve different purposes.

Rejected alternative: relying only on end-to-end tests against a real server makes failures difficult to localize and reproduce.

## 12. Specifications, interoperability, and dependencies

- Record public specifications, public operating-system documentation, and interoperability evidence behind technical decisions.
- Existing clients may be studied for design, implementation, compatibility, and performance information when their licenses and terms permit it.
- Before incorporating external code, verify Apache-2.0 compatibility, attribution requirements, and NOTICE obligations.
- For every new dependency, record its need, alternatives, license, and maintenance status.

Reason: the project should use available implementation knowledge while preserving specification compliance and distributability.

Rejected alternative: importing code or dependencies without verifying origin and license makes redistribution conditions and attack surface unclear.

## 13. Verification

- Daily verification entry point: `./scripts/verify-quick`
- Pre-integration verification entry point: `./scripts/verify`
- Security verification: `./scripts/verify-security`
- Command-line black-box verification: `./scripts/verify-blackbox`
- External interoperability verification: `./scripts/verify-oracle`, enabled explicitly
- Periodic extended verification: `./scripts/verify-nightly`
- Mark work complete and update `docs/progress.md` only after verification succeeds.

## 14. Repository workflows

Use the repeatable workflows under `.agents/skills`.

- `rust-sstp-status`: report repository state and the next work unit
- `rust-sstp-advance-small`: advance the current work unit by one test-driven increment
- `rust-sstp-advance-large`: drive the current work unit to completion
- `rust-sstp-advance-pr`: prepare one reviewable change unit
- `rust-sstp-continue-branch`: resume an existing branch without losing work
- `rust-sstp-review`: review changes against the current work unit
- `rust-sstp-docs-only`: reconcile documentation without changing production code
- `rust-sstp-merge-reviewed`: prepare reviewed and verified work for an authorized integration

Reason: investigation, implementation, verification, and recording should happen in a repeatable order.

Rejected alternative: improvised workflows can omit completion criteria or required verification.

## 15. Final checklist

- [ ] The change stays inside the current PR.
- [ ] Reasons and rejected alternatives are current.
- [ ] Input/output or `unsafe` code has not leaked into `sstp-protocol` or `sstp-session`.
- [ ] The change does not add unjustified primitive exposure or generic dumping-ground modules.
- [ ] Failure cases and regression tests exist where required.
- [ ] Dependency lock files, licenses, and the CVE baseline remain consistent.
- [ ] `./scripts/verify` succeeds.
- [ ] `TODO.md`, `docs/design-todo.md`, and `docs/progress.md` describe the verified state.
