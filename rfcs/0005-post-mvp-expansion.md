# RFC 0005: Post-MVP Expansion Scope

*Status: Active Backlog*

## Summary

This RFC captures features intentionally deferred from MVP (RFC 0004) so scope reduction does not lose design intent. It defines the post-MVP expansion backlog and sequencing constraints.

## Motivation

MVP focuses on proving deterministic install/build/runtime behavior with Tier-1 targets.
Deferred work should remain explicit and tracked as first-class RFC scope.

## Deferred from MVP

### CLI Surface Expansion

- `myx publish`
- `myx list-adapters`
- Optional packaging ergonomics (`pack`, advanced build output options)

### Discovery and Distribution

- Hosted registry API (search, metadata, publish/download endpoints)
- Auth/token model for publish and private registries
- Registry-side package metadata and compatibility indexing

### Runtime Target Expansion (Tier-2)

- `vercel`
- `gemini`
- `claude`

Tier-2 targets must reuse the same profile validation and deterministic loss-report framework from MVP.

### Adapter Model Expansion

- External plugin model for adapters (post built-in-only MVP)
- Adapter capability metadata and conformance levels

### Importer Maturity

- Promote SKILL/OpenAI importers from non-blocking to required release criteria
- Evaluate additional importer targets after Tier-2 export stabilization

### Trust and Integrity Expansion

- Signature/provenance verification model
- Publisher trust roots and policy controls

## Compatibility and Sequencing Constraints

1. Post-MVP additions must not break MVP command behavior or exit code meanings.
2. Hosted registry workflows must preserve local/static-index fallback behavior.
3. New targets must emit deterministic artifacts and structured loss reports.
4. Any plugin model must not weaken policy enforcement guarantees.

## Initial Delivery Order (Post-MVP)

1. Tier-2 export targets (`vercel`, `gemini`, `claude`).
2. Adapter conformance + plugin architecture design.
3. Hosted registry API and publish workflows.
4. Signature and provenance verification.
