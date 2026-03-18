# Permissions

Capability packages can execute arbitrary code, access external services and perform side effects.  To build trust and support safe composition of agent systems, myx requires that every capability declares its permissions up front.  The myx CLI will display these permissions during installation and may refuse to install packages that request unsafe access without explicit user consent.

## Permission Types

The IR defines four categories of permissions:

- **Network** — A list of hostnames the capability may contact.  For example, `api.github.com` or `slack.com`.  A wildcard (`"*"`) indicates unrestricted network access; this should be avoided if possible.
- **Secrets** — Names of environment variables or secret values required by the capability.  Examples include `GITHUB_TOKEN` or `DATABASE_URL`.  During installation, the CLI can prompt the user to provide these secrets or map them from existing credentials.
- **Filesystem** — Paths that the capability may read and/or write.  This may be a specific file (`"/tmp/scratch.txt"`) or a directory.  An empty list implies no filesystem access.  Use with caution.
- **Subprocess** — A boolean flag indicating whether the capability spawns subprocesses (via `exec`, `spawn` or similar).  Subprocesses increase the attack surface and should be used sparingly.

## Enforcement

At the package level, permissions are declarative; myx does not sandbox code execution.  It relies on runtimes (e.g. MCP servers, agent frameworks) to enforce the declared limits.  However, by surfacing the permissions early, myx provides crucial information to developers and operators.  In future versions, myx may integrate with runtime sandboxes to enforce these boundaries automatically.

## Best Practices

- **Minimise**: Declare only the permissions that are actually needed.  For example, if your GitHub capability only reads public repositories, it may not need a secret at all.
- **Principle of least privilege**: Prefer granular hostnames (`api.github.com`) over wildcards (`github.com`), and specific paths over top‑level directories.
- **Transparency**: Document why each permission is needed.  The CLI could surface this documentation when asking the user for consent.