# CONSTITUTION.md — hummbl-forge-engine

**Status:** v0.1
**Steward:** HUMMBL Research Institute
**Approving human:** Reuben Bowlby
**Standard:** HUMMBL Repo Standard v0.1
**Source of record:** git

## 1. Identity

`hummbl-io/hummbl-forge-engine` — Game engine built in Rust for HUMMBL.

- **Class:** library
- **Visibility:** public
- **License:** Apache-2.0

## 2. Scope

This constitution operates under the HUMMBL Repo Standard. This constitution may be stricter, never weaker.

## 3. Protected invariants

1. **Apache-2.0 license.** The license is Apache-2.0, unchanged.
2. **Receipt integrity.** The Krineia chain is append-only and SHA-256-chained.
3. **No secrets.** Never commit credentials, secrets, or runtime tokens.

## 4. Normative files

- `CONSTITUTION.md`
- `KRINEIA.md`
- `AGENTS.md`

## 5. Authority

- **Steward:** HUMMBL Research Institute
- **Approving human:** Reuben Bowlby
- **Agent operating contract:** `AGENTS.md`
- **Receipt manifest:** `KRINEIA.md`

## 6. Receipt-triggering changes

- Any edit to `CONSTITUTION.md`, `KRINEIA.md`, or `AGENTS.md`
- Any change to a protected invariant (Section 3)
- Any release or version bump

## 7. Amendment

Changes to this constitution require: a PR, a KRINEIA receipt, and human approval (Reuben Bowlby).
