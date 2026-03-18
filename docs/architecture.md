# Architecture

The myx architecture is designed around a central, runtime‑agnostic capability model and a flexible adapter system.  This section describes the main layers and how data flows through them.

## Layers

myx consists of four conceptual layers:

1. **Source Formats** – Capabilities originate in many formats: SKILL.md folders, MCP servers, OpenAI tool schemas, LangChain tool classes, bespoke scripts and so on.  These formats are treated as *inputs*.
2. **Import Adapters** – For each supported source format, myx implements an adapter that detects and parses the input, producing a canonical *Capability IR* instance.  Adapters are small modules with a uniform interface: `detect()` to determine applicability and `import()` to perform the conversion.
3. **Capability IR** – The canonical JSON schema that captures all information about a capability: identity, metadata, instructions, tool definitions, permissions, runtime entrypoints and compatibility hints.  Once a capability is represented in this intermediate form, it can be serialised to disk, inspected or versioned.
4. **Export Adapters** – To use a capability with a particular runtime, an export adapter turns the IR into the appropriate artefact.  Examples include OpenAI tool definition files, SKILL.md bundles, MCP server configuration and custom plugin formats.  The export adapter interface is symmetrical to import: `export()` takes an IR instance and a destination directory and writes the necessary files.

## Data Flow

```text
          Source Format                 Capability IR               Runtime Artefact
  ────────────────────────────┐ ┌───────────────────────────┐ ┌───────────────────────┐
  SKILL.md folder            │ │                           │ │ SKILL.md bundle       │
  MCP server descriptor  ────▶│ Import Adapter ────────────▶│ Export Adapter ───────▶
  Tool schema JSON           │ │                           │ │ OpenAI tools JSON     │
  LangChain Python class     │ │                           │ │ MCP server config     │
  (others)                   │ │                           │ │ (others)              │
  ────────────────────────────┘ └───────────────────────────┘ └───────────────────────┘
```

1. A capability in some source format is passed to the corresponding import adapter.
2. The adapter produces a capability represented in the intermediate JSON schema.
3. The IR can be inspected, saved, versioned and published.
4. When needed, an export adapter turns the IR into files consumable by a specific runtime.

## Why this approach?

By isolating the core model (the IR) from the specifics of source and target formats, myx can support new formats with minimal changes.  Adding support for a new runtime means writing one export adapter; adding support for a new existing format means writing one import adapter.  Everything else (versioning, permissions, registry, CLI) stays the same.