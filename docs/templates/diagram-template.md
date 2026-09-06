# Mermaid Diagram Template (Dark Theme)

This is a copy-pasteable boilerplate template for new Mermaid diagrams, configured with the gentle RGB dark base theme.

```mermaid
%%{init: {
  'theme': 'base',
  'themeVariables': {
    'darkMode': true,
    'background': '#161922',
    'mainBkg': '#1e2230',
    'nodeBorder': '#434c5e',
    'textColor': '#e2e8f0',
    'fontFamily': 'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    'fontSize': '14px',
    'lineColor': '#8892b0',
    'primaryColor': '#422026',
    'primaryTextColor': '#fde8ec',
    'primaryBorderColor': '#e06c75',
    'secondaryColor': '#1b3528',
    'secondaryTextColor': '#e6f7ee',
    'secondaryBorderColor': '#73c991',
    'tertiaryColor': '#1d2c44',
    'tertiaryTextColor': '#e4f0fc',
    'tertiaryBorderColor': '#61afef',
    'clusterBkg': '#13161f',
    'clusterBorder': '#373e51',
    'noteBkgColor': '#2e271a',
    'noteTextColor': '#fdf4db',
    'noteBorderColor': '#e5c07b',
    'edgeLabelBackground': '#1a1d27',
    'actorBkg': '#1e2230',
    'actorBorder': '#61afef',
    'actorTextColor': '#e2e8f0',
    'actorLineColor': '#6c7693',
    'signalColor': '#8892b0',
    'signalTextColor': '#e2e8f0',
    'altBackground': '#13161f',
    'activationBkgColor': '#2d3548',
    'activationBorderColor': '#61afef'
  }
}}%%
flowchart TD
    classDef primary fill:#422026,stroke:#e06c75,stroke-width:1.5px,color:#fde8ec;
    classDef secondary fill:#1b3528,stroke:#73c991,stroke-width:1.5px,color:#e6f7ee;
    classDef tertiary fill:#1d2c44,stroke:#61afef,stroke-width:1.5px,color:#e4f0fc;
    classDef note fill:#2e271a,stroke:#e5c07b,stroke-width:1.5px,color:#fdf4db;

    subgraph Scope["System / Component Scope"]
        A["🔴 Primary (Core Domain / Critical / Invariant)"]:::primary
        B["🟢 Secondary (Service / Logic / Processing)"]:::secondary
        C[("🔵 Tertiary (Storage / Adapter / External)")]:::tertiary

        A -->|"Refined by / Invokes"| B
        B <-->|"Persists to / Queries"| C
    end

    N1["📌 Callout / Constraint Note"]:::note
    A -.-> N1
```

## Quick Specialization Guide

1. **Semantic Roles**:
   - `:::primary` (Red): Core domain logic, invariants, critical root components, entrypoints.
   - `:::secondary` (Green): Application services, processing pipelines, transformers, workers.
   - `:::tertiary` (Blue): Datastores, caches, external APIs, adapters, I/O boundaries.
   - `:::note` (Amber): Security constraints, validation guards, architectural callouts.
2. **Layout Direction**:
   - Change `flowchart TD` (top-to-bottom) to `flowchart LR` (left-to-right) for data pipelines.
3. **Labels**:
   - Always wrap node labels in double quotes (`"..."`) when using parentheses, colons, or HTML tags (e.g. `<br/>` for multiline text).

For full details, color palette breakdown, specialization instructions, and diagram type examples (Sequence, State, Class, Architecture), see the [Mermaid Style Guide](mermaid-style-guide.md).
