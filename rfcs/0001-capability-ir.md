# RFC 0001: Capability IR

*Status: Accepted*

## Summary

This RFC proposes a canonical intermediate representation (IR) for agent capabilities.  The IR serves as the single source of truth for packaging, inspecting and exporting capabilities across multiple runtimes.  Its design emphasises portability, transparency and extensibility.

## Motivation

In the current ecosystem, capabilities are defined in numerous ad‑hoc formats.  There is no consistent way to package a capability, declare its permissions or reuse it across frameworks.  A canonical IR allows myx to bridge these formats by providing a common representation that importers can produce and exporters can consume.

## Specification

The IR is a JSON object with a fixed set of top‑level fields.  See `docs/capability-ir.md` for the full schema.  Key points include:

- **Identity:** name, version, publisher and license.
- **Metadata:** description, homepage, source repository.
- **Capabilities:** human‑readable labels describing the actions provided.
- **Instructions:** system and usage prompts to instruct the agent.
- **Tools:** structured definitions of callable functions, including JSON Schema parameters.
- **Permissions:** declarations of network hosts, secrets, filesystem access and subprocess usage.
- **Runtime entrypoints:** references to scripts or servers for long‑running capabilities.
- **Compatibility:** hints indicating which runtimes and platforms are supported.

### Versioning

The IR schema itself has a `schema_version` field.  Changes to the schema are governed by semantic versioning:

- Patch (e.g. 0.1.x) – backwards‑compatible clarifications or additional optional fields.
- Minor (e.g. 0.x) – backwards‑compatible additions (new sections or fields).
- Major (e.g. 1.0) – breaking changes.  Tools must detect and migrate accordingly.

## Drawbacks

- Locking down the IR structure early may restrict innovation in capability modelling.  However, the schema is intentionally flexible and can be extended through minor version bumps.
- Converting very complex capabilities into a static IR may require compromises or approximations.

## Alternatives

Alternatives considered:

- **No IR:** Use separate package formats for each runtime and rely on direct conversion between them.  This leads to a combinatorial explosion of converters.
- **Single universal format (e.g. SKILL.md only):** While appealing, SKILL.md lacks some features (e.g. structured permission declarations) and is tied to one ecosystem.

## Unresolved Questions

- Should the IR support multi‑language prompts or internationalisation?
- How should complex dependency graphs between capabilities be represented?
- Are there additional permission categories (e.g. GPUs, sensors) needed for specialised capabilities?