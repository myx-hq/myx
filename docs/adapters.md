# Adapters

Adapters are the key extensibility mechanism in myx.  They allow the system to ingest capabilities from many different formats (imports) and to export the canonical capability model to many different runtimes (exports).  Each adapter is an independent module implementing a simple interface so that new formats can be added without changing the core library.

## Import Adapters

An **import adapter** is responsible for detecting and converting a capability in a source format into the canonical Capability IR.  Each adapter exposes two functions:

- `detect(path: string) → boolean` — returns true if the file or directory at `path` appears to be in the adapter’s supported format.  For example, the SKILL.md importer might check for the existence of a file called `SKILL.md`.
- `import(path: string) → CapabilityIR` — reads the source file(s) and produces a JavaScript/JSON representation of the capability IR.  If the import fails due to unsupported features or malformed input, the adapter should throw an informative error.

Import adapters should avoid embedding runtime‑specific assumptions.  Their job is to faithfully represent the semantics of the source format in the IR.  Where the source format lacks certain information (e.g. missing permissions), adapters may supply sensible defaults or leave fields blank.

## Export Adapters

An **export adapter** takes a Capability IR instance and a destination directory, then writes out the files required for a specific runtime.  Each adapter exposes a single function:

- `export(capability: CapabilityIR, outputPath: string): void` — generates runtime artefacts at `outputPath`.  The format of these artefacts varies by runtime; for example:
  - **OpenAI tools exporter** writes a `.tools.json` file and a companion `.instructions.md` file containing prompt text.
  - **SKILL.md exporter** writes a `SKILL.md` file and supporting files such as a commands table.
  - **MCP exporter** writes a `server.json` or `server.js` config needed to register the capability as an MCP server.

Export adapters are free to perform additional validation on the IR and may emit warnings if some features are unsupported by the target runtime.  They should not mutate the IR itself.

## Implementing Adapters

Adapters can be written in any language supported by the myx CLI (initially TypeScript/Node.js).  To be recognised by the CLI, an adapter must register itself in a discovery mechanism (e.g. by exporting certain symbols or by following a naming convention).  The CLI will iterate over available import adapters to determine which one matches a given source directory, and over export adapters to list available output formats.

## Contributing New Adapters

If you wish to add support for a new format or runtime, consider the following steps:

1. Examine existing adapters for examples of detection and conversion logic.
2. Write a new module implementing the adapter interface.
3. Add tests using sample capabilities in the new format.
4. Update the documentation and open an RFC if the new adapter requires changes to the IR schema.