# RFC 0009: Run Command Spec

*Status: Draft*

## Purpose

`myx run` provides a minimal execution path to prove runtime and policy behavior end-to-end.

## Command

```bash
myx run <package>.<tool> [--input json]
```

## Examples

```bash
myx run github.search_repositories --input '{"query":"rust"}'
myx run local.echo --input '{"text":"hello"}'
```

## Flow

```text
resolve package
-> load profile
-> validate tool exists
-> construct execution request
-> enforce policy
-> execute via runtime
-> return output
```

## Input Contract

- `--input` accepts a JSON object
- input must validate against the tool parameter schema
- invalid input returns deterministic validation error

## Default Human Output

```text
✓ executed github.search_repositories
-> 12 results returned
```

## JSON Output

```json
{
  "status": "success",
  "duration_ms": 120,
  "output": {
    "results": 12
  }
}
```

## Error Output

```json
{
  "status": "error",
  "error": "E_TOOL_NOT_FOUND",
  "message": "Tool 'search_repositories' not found in package 'github'"
}
```

## Error Codes

- `E_TOOL_NOT_FOUND`
- `E_INPUT_INVALID`
- `E_POLICY_DENIED`
- `E_EXEC_FAIL`
- `E_TIMEOUT`
- `E_SECRET_MISSING`

## MVP Requirement

A valid MVP should support at least:

- one HTTP-based tool path
- one subprocess-based tool path

through `myx run`.
