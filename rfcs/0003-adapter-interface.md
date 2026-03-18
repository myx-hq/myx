# RFC 0003: Adapter Interface

*Status: Draft*

## Summary

This RFC defines the minimal interfaces for import and export adapters in the myx ecosystem.  A clear contract makes it easy for third parties to add support for new source formats and target runtimes without modifying the core library.

## Motivation

The flexibility of myx hinges on its adapter system.  Adapters allow the CLI to ingest capability definitions from diverse formats and output runtime‑specific artefacts.  A lightweight, language‑agnostic interface enables rapid expansion of supported ecosystems.

## Import Adapter Interface

An import adapter is a module that exports two functions:

```ts
/**
 * Determines whether the adapter can handle the given path.
 * Should return true if `path` contains a capability in this format.
 */
export function detect(path: string): Promise<boolean>;

/**
 * Converts the capability at `path` into the canonical IR.
 * Throws an error if the input is malformed or unsupported.
 */
export function importCapability(path: string): Promise<CapabilityIR>;
```

Guidelines:

- `detect()` must not perform heavy operations; it should quickly identify whether the directory contains a recognisable file (e.g. `SKILL.md`, `server.json`).
- `importCapability()` should read the necessary files and populate all IR fields.  If some fields cannot be inferred, it may leave them blank or apply defaults.
- If multiple import adapters can handle the same folder, the CLI will apply them in order of specificity (to be defined).

## Export Adapter Interface

An export adapter exports a single function:

```ts
/**
 * Writes runtime artefacts for the given capability IR into `outputPath`.
 * May throw if the IR contains unsupported features.
 */
export function exportCapability(capability: CapabilityIR, outputPath: string): Promise<void>;
```

Guidelines:

- The adapter should validate the IR against any requirements of the target runtime (e.g. parameter types supported by OpenAI tools).
- It should write files into the `outputPath` directory, creating subdirectories as needed (e.g. `openai`, `skill`, etc.).
- It must not modify the passed IR.

## Adapter Discovery

To simplify usage, adapters may be discovered via a naming convention (e.g. files in a particular folder ending with `-importer.ts` or `-exporter.ts`) or via explicit registration in a configuration file.  The CLI should provide commands such as `myx list-adapters` to list available import and export adapters.

## Open Questions

- Should adapters support streaming conversion for large packages?
- How should adapters handle partial or experimental support for features?  Should they emit warnings or errors?
- What is the best way to support adapters in multiple languages (e.g. Python)?