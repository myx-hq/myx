# Spec Workplan

This document turns the current roadmap into an executable spec track for the pre-implementation phase. The objective is to remove ambiguity before CLI and adapter code starts.

## Scope

The workplan covers:

- Capability and package contracts.
- CLI behavior and package lifecycle semantics.
- Adapter conformance requirements.
- Registry API and publish/install guarantees.
- Security and trust model requirements.

It does not cover implementation planning details such as sprint assignment or language-level module layout.

## Working Rules

1. Every normative behavior should live in exactly one canonical RFC section.
2. Example files in `examples/` must stay aligned with active RFCs.
3. A roadmap milestone is only considered complete when the RFC is at least `Proposed`, an example exists for the feature, and acceptance criteria are testable by a CLI prototype.
4. Conflicts between docs and RFCs are resolved in favor of RFC text.

## Spec Sequence

## Track A: Core Model and Package Contract

Status: In progress

- RFC 0001: Capability IR (draft exists)
- RFC 0002: Package Format (draft exists)
- Gap to close: explicitly define required vs optional manifest fields and path resolution rules.

Definition of done:

- A validator can deterministically accept or reject a package without runtime-specific logic.
- `examples/github-capability/` contains both `myx.yaml` and `capability.json` that validate against RFCs.

## Track B: CLI Behavioral Contract

Status: Started (this repo now includes RFC 0004 draft)

- RFC 0004: CLI command semantics, output contract, exit codes, lockfile behavior.
- RFC 0005: Post-MVP expansion scope for deferred command/registry/target features.

Definition of done:

- `myx init/add/inspect/build` have unambiguous success/failure behavior.
- Non-interactive and CI-safe behavior is defined.
- Lockfile update semantics are deterministic.

## Track C: Adapter Conformance

Status: Planned

- Expand RFC 0003 with conformance levels and required diagnostics.
- Define adapter capability matrix and fallback behavior for lossy conversions.

Definition of done:

- A third-party adapter author can implement against a normative checklist.
- Import and export adapters have shared error categories and warning structure.

## Track D: Registry Contract

Status: Planned

- Promote `docs/registry.md` into a normative RFC with wire formats.
- Define package immutability, checksum/signature fields, and auth expectations.

Definition of done:

- CLI can resolve package metadata from any compliant registry.
- Package identity and integrity checks are fully specified.

## Track E: Security and Trust

Status: Planned

- Define permission policy levels (permissive, review-required, strict).
- Define signature trust roots and verification outcomes.
- Define install-time policy evaluation model.

Definition of done:

- Install decisions are deterministic from package metadata plus policy config.
- Security warnings and hard failures are clearly differentiated.

## Immediate Next RFCs

After RFC 0005, highest leverage order:

1. RFC 0006: Lockfile Format and Resolution Algorithm.
2. RFC 0007: Adapter Conformance Levels and Diagnostics.
3. RFC 0008: Registry API v1.
4. RFC 0009: Security Policy Evaluation and Signature Verification.
