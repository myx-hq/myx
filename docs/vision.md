# Vision

The long‑term vision for **myx** is to create a vibrant, interoperable ecosystem for agent capabilities, analogous to the impact that package managers like npm or pip have had on software development.  In the current landscape, a capability’s usefulness is often tied to the agent framework in which it was built; portability is an afterthought.  myx aims to break down those walls by embracing a few core principles:

1. **Neutral infrastructure.**  myx does not compete with agent frameworks.  It focuses on packaging, versioning and interoperability, while existing runtimes continue to innovate on reasoning, planning and execution.
2. **Single source of truth.**  A capability should be authored once in a canonical form (the Capability IR).  From that, adapters can produce runtime artefacts for many frameworks, ensuring consistency.
3. **Transparency and safety.**  Capabilities must declare the permissions they require (network hosts, secrets, filesystem access, etc.) and be auditable before use.  This fosters trust and minimises the risk of malicious or poorly scoped actions.
4. **Community driven.**  A healthy ecosystem depends on contributions from developers.  myx will encourage community packages, robust documentation, and an open governance model for evolving the spec.

By focusing on these principles, myx aims to become the “package manager for intelligence”, enabling developers to compose sophisticated agents from modular, auditable capabilities that run on any compatible runtime.