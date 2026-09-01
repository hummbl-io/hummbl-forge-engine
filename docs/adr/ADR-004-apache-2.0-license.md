# ADR-004: Apache 2.0 License

**Status:** Approved (operator-approved 2026-09-01)
**Date:** 2026-09-01

## Context

Forge Engine needs a license. This is the operator's decision — agents do
not choose licenses.

## Decision

License the Forge Engine code under **Apache License 2.0**.

## Rationale

1. **Operator decision.** The operator selected Apache 2.0 from the proposed
   options (MIT, Apache 2.0, Source-available/BSL, Decide later).

2. **Compatible with Bevy.** Bevy is MIT-licensed, which is compatible with
   Apache 2.0. We can use and distribute Bevy without conflict.

3. **Patent grant.** Apache 2.0 includes an explicit patent grant, which
   MIT does not. This provides additional legal protection for contributors
   and users.

4. **Industry standard.** Apache 2.0 is well-understood, widely used, and
   trusted by both open-source communities and commercial entities.

5. **Permissive.** Allows anyone to use, modify, and distribute the code
   with attribution. Maximizes potential adoption while retaining the
   patent protection.

## Consequences

- Anyone can use Forge Engine code under the Apache 2.0 terms.
- Derivative works must retain the license and copyright notice.
- Patent grants apply to contributors and users.

## Alternatives considered

- **MIT:** Simpler but no patent grant. Operator preferred Apache 2.0.
- **Source-available / BSL:** Would restrict commercial use and community
   adoption. Operator chose permissive.
- **Decide later:** Valid option, but operator chose to decide now.
