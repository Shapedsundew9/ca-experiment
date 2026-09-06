# Rust & Python Agentic Engineering Template

> A template and multi-agent orchestration framework for specification-driven systems development in Rust and Python.

---

## 🌟 Overview

This repository is a **starter template** designed for teams and engineers building software systems using AI-assisted, specification-first workflows. It combines a dual-language runtime environment (**Rust** for high-performance, memory-safe core logic; **Python** for rapid tooling, analysis, and data scripts) with a suite of specialized **GitHub Copilot / Antigravity Agent Orchestrators**.

The agent suite is structured into three strictly decoupled, domain-bounded tracks:

1. **Specification & Architecture Track (`Spec:*`)**: Progressively refines human intent into formal domain invariants, user experience journeys, logical interface contracts, and technology realization profiles with explicit human-in-the-loop decision gates.
2. **Implementation & Quality Track (`Code:*`)**: Operates on a strict **Repeat Until Good (RUG)** protocol—decomposing tasks, dispatching implementation workers, and subjecting every change to multi-tiered automated verification.
3. **Scientific Research Track (`Sci:*`)**: Drives hypothesis formulation, rigorous empirical experiment protocols, and dynamical systems diagnostics, translating abstract research designs into standardized experiment implementation specifications via a streamlined 2-subagent execution model.

---

## 🏛️ Architectural Guardrails: Depth & Domain Boundaries

To prevent recursion, runaway execution, and architectural confusion across platforms (including GitHub Copilot in VS Code and Google Antigravity `agy`), the multi-agent system enforces foundational design constraints:

### 1. The 1-Level Subagent Depth Constraint

In modern coding agent platforms, subagent delegation is strictly **one level deep**:

$$\text{Operator / User} \longrightarrow \text{Lead Orchestrator} \longrightarrow \text{Leaf Subagent}$$

- **No Subagents of Subagents**: Subagents **cannot** invoke further subagents. Nested delegation is disabled or blocked by default across platforms (such as VS Code Copilot agent mode and Antigravity CLI) to eliminate the risk of unbounded recursion, exponential token cost escalation ($O(B^D)$), lossy semantic compression ("the telephone game" across summarization layers), and runaway execution loops.
- **Orchestrators Cannot Run Inside Orchestrators**: An orchestrator requires root management privileges to decompose tasks, dispatch child workers, and maintain session tracking. Running an orchestrator inside another orchestrator causes immediate delegation failure or permission denial.
- **The Operator is the Root Lifecycle Coordinator**: For cross-track handoffs that transition between major software phases (e.g. from formal specification to production coding), the **Operator / User** acts as the overarching coordinator, verifying stage gates and invoking the appropriate lead orchestrator.

### 2. Tailored Orchestration Styles for Different Engineering Intents

Different software workflows have fundamentally divergent operational goals. This template provides three distinct orchestration tracks, each designed for its specific domain:

| Track | Engineering Intent | Workflow Nature | Verification Model | Subagent Topology |
| :--- | :--- | :--- | :--- | :--- |
| **Specification Track (`Spec:*`)** | Formal domain invariants, OpenAPI schemas, ADRs, realization profiles | **Convergent & Deliberate**: Infrequent, multi-tier refinement | Explicit human gates (Gate 0, UX, Gate 1, Gate 2) | 1-level: `Spec: Orchestrator` $\rightarrow$ drafting & auditing specialists |
| **Production Code Track (`Code:*`)** | Memory-safe Rust core, production services, clean APIs, database migrations | **Convergent & Zero-Drift**: Enterprise software engineering | Strict **Repeat Until Good (RUG)**, independent QA, complete test suites (`cargo test`, `unittest`) | 1-level: `Code: RUG Orchestrator` $\rightarrow$ `swe`, `debug`, `qa-lite`, `qa` |
| **Scientific Research Track (`Sci:*`)** | Hypothesis testing, parameter sweeps, dynamical diagnostics, discovery loops | **Divergent & Evolutionary**: Rapid iteration, controlled exploration ("wandering") | Pre-registered metrics, automated telemetry reduction, Gate H/P & Gate I steering | 1-level: `Sci: Orchestrator` $\rightarrow$ `Sci: Theory & Protocol`, `Sci: Execution & Analysis` |

#### Resolving the Science-to-Code Orchestration Dilemma

A common design pitfall in multi-agent systems is attempting to invoke `Code: RUG Orchestrator` *inside* `Sci: Orchestrator` for exploratory script tweaks. This fails on two counts:

1. **Nested Subagent Violation**: It creates an illegal 2-level delegation chain (`Sci: Orchestrator` $\rightarrow$ `RUG Orchestrator` $\rightarrow$ `SWE` worker).
2. **Impedance Mismatch**: RUG is built for enterprise release rigor (strict task decomposition, independent diff audits, acceptance checklists). Scientific exploration requires an **agile lab technician**: packaging an isolated experiment, exploring parameter spaces with adaptive intelligence, reducing telemetry, and feeding results back into the diagnostic state machine.

To solve this, **`Sci: Orchestrator` directly dispatches `Sci: Execution & Analysis`** within its own 1-level hierarchy as its execution worker. The Scientific Research Track is autonomous and self-contained; its mission terminates not in production software releases, but in rigorous empirical discovery, reproducible experiment packages, execution manifests, and durable diagnostic dossiers.

### 3. How to Choose and Use Flows in Your Project

Depending on the primary intent of your project or sub-campaign, select the appropriate track:

- **Track 1: Production Systems Engineering (Spec $\rightarrow$ Code)**:
  Use when building robust libraries, services, or tools. Run `Spec: Orchestrator` to produce formal requirements in `docs/requirements/`. Once Gate 2 is approved, launch `Code: RUG Orchestrator` pointing to the approved spec.
- **Track 2: Autonomous Scientific Discovery (Sci Loop)**:
  Use for machine learning experiments, evolutionary algorithms, dynamical simulations, or complex systems discovery. Run `Sci: Orchestrator`. It directs theoretical hypotheses, designs protocols, provisions isolated experiment packages in language-scoped directories (`python/experiments/`), conducts intelligent parameter exploration, reduces telemetry, and iterates based on diagnostic evaluations—recording permanent, reproducible findings under Git tags.

---

## 🤖 Multi-Agent System Architecture

The following diagram illustrates how the three decoupled agent tracks operate within the 1-level depth constraint, interact with the user, and exchange durable artifacts:

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
    'edgeLabelBackground': '#1a1d27'
  }
}}%%
flowchart TB
    classDef primary fill:#422026,stroke:#e06c75,stroke-width:1.5px,color:#fde8ec;
    classDef secondary fill:#1b3528,stroke:#73c991,stroke-width:1.5px,color:#e6f7ee;
    classDef tertiary fill:#1d2c44,stroke:#61afef,stroke-width:1.5px,color:#e4f0fc;
    classDef note fill:#2e271a,stroke:#e5c07b,stroke-width:1.5px,color:#fdf4db;

    USER["👤 Operator / User<br/><i>(Root Lifecycle Coordinator & Principal Investigator)</i>"]:::note

    subgraph SPEC_TRACK["Specification & Architecture Track (Spec:*)"]
        SPEC_O["🎯 Spec: Orchestrator<br/><i>(Progressive Refinement)</i>"]:::primary
        SPEC_SUBS["👥 Spec Subagents<br/><i>(PRD, PM, Req, UX, API, Arch, ADR)</i>"]:::secondary
        SPEC_ARTS["📄 Specification Artifacts<br/><i>(docs/requirements/, docs/architecture/)</i>"]:::tertiary
        SPEC_O <-->|"1-level dispatch"| SPEC_SUBS
        SPEC_SUBS -->|"Generates"| SPEC_ARTS
    end

    subgraph CODE_TRACK["Implementation & Quality Track (Code:*)"]
        CODE_O["🎯 Code: RUG Orchestrator<br/><i>(Repeat-Until-Good Manager)</i>"]:::primary
        CODE_SUBS["👥 Code Subagents<br/><i>(SWE, Debug, QA Lite, QA, Security)</i>"]:::secondary
        CODE_PROD["💻 Production Code & Tests<br/><i>(src/, python/src/tools/, tests/)</i>"]:::tertiary
        CODE_O <-->|"1-level dispatch"| CODE_SUBS
        CODE_SUBS -->|"Implements & Verifies"| CODE_PROD
    end

    subgraph SCI_TRACK["Scientific Research Track (Sci:*)"]
        SCI_O["🎯 Sci: Orchestrator<br/><i>(Lead Scientist)</i>"]:::primary
        SCI_THEORY["📐 Sci: Theory & Protocol<br/><i>(Hypotheses & Protocols)</i>"]:::secondary
        SCI_EXEC["💻 Sci: Execution & Analysis<br/><i>(Provisions, runs, analyzes)</i>"]:::secondary
        SCI_ARTS["🔬 Research Artifacts & Telemetry<br/><i>(Protocols, Manifests, data/telemetry/)</i>"]:::tertiary
        SCI_O <-->|"1-level dispatch"| SCI_THEORY
        SCI_O <-->|"1-level dispatch"| SCI_EXEC
        SCI_EXEC -->|"Provisions, runs, analyzes"| SCI_ARTS
        SCI_THEORY -->|"Generates"| SCI_ARTS
    end

    USER -->|"1. Directs Intent & Approves Tiers"| SPEC_O
    SPEC_ARTS -.->|"Approved Specs Handed to User"| USER
    USER -->|"2. Dispatches Specs for Implementation"| CODE_O
    CODE_PROD -.->|"Production Release Handed to User"| USER
    CODE_O -.->|"3. Spec Change Proposal (Reverse Escalation)"| USER
    USER -.->|"4. Amends Specs on Blocker"| SPEC_O

    USER <-->|"A. Directs Research Campaigns (Gates H/P & I)"| SCI_O
    SCI_ARTS -.->|"B. Validated Findings & Permanent Research Dossier"| USER
```

---

## 📐 Specification & Architecture Track (`Spec:*`)

The **Specification Track** governs the transition from ambiguous human requirements to verifiable software contracts. Led by [`Spec: Orchestrator`](.github/agents/spec-orchestrator.agent.md), it enforces:

- **Pure Specification Scope**: Focuses strictly on problem framing, domain modeling, interface schemas, and realization constraints. Never writes executable runtime code.
- **Persistent State Across Sessions**: Maintains a durable specification tracker in `docs/requirements/ROADMAP.md` (instantiated from `docs/templates/spec-roadmap-template.md`), capturing stage completions, gate timestamps, approved requirement ranges, and open forks.
- **In-Domain Quality Audits**: Auditing is performed within the track (`Spec: Product Manager` audits business value and scope discipline; `Spec: Architecture Reviewer` verifies system boundaries, security, and schema rigor). Auditors generate durable reports in `docs/requirements/audits/AUDIT-T{tier}-*.md`.
- **ADR Governance & Central Registry**: Evaluates architectural fork points via [`Spec: ADR Generator`](.github/agents/spec-adr-generator.agent.md), registers them in [`docs/architecture/README.md`](docs/architecture/README.md), and promotes them from `Proposed` to `Accepted` upon user sign-off at Gate 1 or Gate 2.
- **Human Decision Gates & Invalidation Discipline**: Progresses only upon explicit operator sign-off against structured gate verification checklists. If revisions occur, downstream artifacts are invalidated before re-drafting.

### Specification Refinement Workflow

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
    'edgeLabelBackground': '#1a1d27'
  }
}}%%
flowchart TD
    classDef primary fill:#422026,stroke:#e06c75,stroke-width:1.5px,color:#fde8ec;
    classDef secondary fill:#1b3528,stroke:#73c991,stroke-width:1.5px,color:#e6f7ee;
    classDef tertiary fill:#1d2c44,stroke:#61afef,stroke-width:1.5px,color:#e4f0fc;
    classDef note fill:#2e271a,stroke:#e5c07b,stroke-width:1.5px,color:#fdf4db;

    ORCH["🎯 Spec: Orchestrator<br/><i>(Align, Draft, Audit, Gate)</i>"]:::primary
    ROADMAP[("📋 docs/requirements/ROADMAP.md<br/><i>(Persistent State Tracker)</i>")]:::tertiary

    subgraph T0_STAGE["Tier 0: Domain Invariants & Product Goals (Tech-Agnostic)"]
        PRD["Spec: PRD<br/><i>(Sole Drafting Author: prd.md synthesis)</i>"]:::secondary
        PM_AUDIT["Spec: Product Manager<br/><i>(Auditor: Business value, KPI validity & backlog slicing)</i>"]:::secondary
        ARCH_AUDIT["Spec: Architecture Reviewer<br/><i>(Auditor: Invariant testability & boundaries)</i>"]:::secondary
        SPEC0["Spec: Specification<br/><i>(Extractor: Formal REQ-T0 in EARS syntax)</i>"]:::secondary
        GATE0["🔒 Gate 0: Scope & Invariants Sign-Off"]:::note
    end

    subgraph UX_STAGE["UX Track: User Experience & Workflows"]
        UX["🎨 Spec: UX Designer<br/><i>(JTBD, user journeys & interaction flows)</i>"]:::secondary
        AUDIT_UX["Spec: Product Manager<br/><i>(Audit: Flow ergonomics & edge states)</i>"]:::secondary
        GATE_UX["🔒 Gate UX: Interaction Sign-Off"]:::note
    end

    subgraph T1_STAGE["Tier 1: Logical Architecture & Interface Contracts"]
        ARCH["Spec: Architecture Reviewer<br/><i>(Mandatory: Component boundaries & patterns)</i>"]:::secondary
        API["Spec: API Architect<br/><i>(Mandatory: OpenAPI, schemas & resilience contracts)</i>"]:::secondary
        ADR["Spec: ADR Generator<br/><i>(Fork point trade-off matrices)</i>"]:::secondary
        AUDIT_T1["Spec: Architecture Reviewer<br/><i>(Audit: System cohesion & schema rigor)</i>"]:::secondary
        GATE1["🔒 Gate 1: Contract Sign-Off & ADR Promotion"]:::note
    end

    subgraph T2_STAGE["Tier 2: Technology Realization Profiles"]
        SPEC2["Spec: Specification<br/><i>(Mandatory: REQ-T2 realization profiles)</i>"]:::secondary
        AUDIT_T2["Spec: Architecture Reviewer<br/><i>(Audit: Traceability & constraint proof)</i>"]:::secondary
        GATE2["🔒 Gate 2: Implementation Readiness Sign-Off"]:::note
    end

    ORCH -->|"1. Initialize / Resume"| ROADMAP
    ORCH -->|"2. Dispatch drafting"| PRD
    PRD -->|"Draft prd.md"| PM_AUDIT & ARCH_AUDIT
    PM_AUDIT & ARCH_AUDIT -->|"Validated Proposal & Audit Log"| SPEC0
    SPEC0 -->|"Formal Requirements Package"| GATE0
    PM_AUDIT -.->|"Audit Failure: Re-draft"| PRD
    ARCH_AUDIT -.->|"Audit Failure: Re-draft"| PRD

    GATE0 -.->|"Update State"| ROADMAP
    GATE0 -->|"Approved: Slices Issues"| PM_AUDIT
    GATE0 -->|"Operator Prompt"| UX_DECISION{"Has User/CLI Interaction?"}:::primary
    UX_DECISION -->|"Yes"| UX
    UX_DECISION -->|"No (Skip UX)"| T1_STAGE

    UX -->|"Journey Artifacts"| AUDIT_UX
    AUDIT_UX -->|"Verified UX & Audit Log"| GATE_UX
    GATE_UX -.->|"Update State"| ROADMAP
    GATE_UX -->|"Approved"| T1_STAGE

    T1_STAGE --> ARCH & API
    ARCH -.->|"Architectural Fork Detected"| ADR
    ARCH & API & ADR --> AUDIT_T1
    AUDIT_T1 -->|"Verified Contracts & Audit Log"| GATE1
    GATE1 -.->|"Update State & Accept ADRs"| ROADMAP
    AUDIT_T1 -.->|"Audit Failure: Re-draft"| ARCH

    GATE1 -->|"Approved"| T2_STAGE
    T2_STAGE --> SPEC2
    SPEC2 --> AUDIT_T2
    AUDIT_T2 -->|"Verified Realization & Audit Log"| GATE2
    GATE2 -.->|"Release Package"| ROADMAP
    GATE2 -->|"Approved"| FINAL_ARTIFACTS["📦 Approved Requirements Package<br/><i>(docs/requirements/, docs/architecture/)</i>"]:::tertiary

    GATE1 -.->|"Gate 1 Revision / Reject"| T0_STAGE
    GATE2 -.->|"Gate 2 Revision / Reject"| T1_STAGE
```

### `Spec:*` Subagents Catalog

| Agent Name | Role | Call Nature | Inputs | Primary Deliverables |
| :--- | :--- | :--- | :--- | :--- |
| **`Spec: Orchestrator`** | Master Specification Manager | **Orchestrator** | User goal, roadmap | Decomposed plan, persistent `ROADMAP.md`, stage gates, ADR promotions |
| **`Spec: PRD`** | Sole Tier 0 Drafting Author | **Mandatory (Tier 0 Author)** | User intent, scope | `prd.md` (10-section outline, personas, user stories) |
| **`Spec: Product Manager`** | Value & Scope Auditor, Backlog Slicer | **Mandatory (Tier 0 Auditor)** | Draft PRD, feature scope | Value critique report, audit log (`AUDIT-T0-*`), sized GitHub issues & epics |
| **`Spec: Specification`** | Formal Requirements Author | **Mandatory (T0 & T2)** | PRDs, logical designs | `docs/requirements/` in r9ts format (`REQ-T0-*`, `REQ-T2-*`) |
| **`Spec: UX Designer`** | Experience & Workflow Designer | **Optional (Gate 0 Decision)** | User workflows | JTBD analysis, user journey maps, CLI/Web interaction flows |
| **`Spec: API Architect`** | Interface & Contract Designer | **Mandatory (Tier 1)** | Component boundaries | OpenAPI 3.1/JSON schemas, error taxonomies, resilience SLA budgets (NO runtime code) |
| **`Spec: Architecture Reviewer`** | Architecture & Security Reviewer | **Mandatory (Tier 1 & Audits)** | System designs, NFRs | Component decomposition, security architecture, audit logs (`AUDIT-T1-*`, `AUDIT-T2-*`) |
| **`Spec: ADR Generator`** | Architecture Decision Author | **Optional / Triggered** | Technology trade-offs | `docs/architecture/adr-NNNN-[slug].md` records & `docs/architecture/README.md` registry |

---

## ⚡ Implementation & Quality Track (`Code:*`)

The **Implementation Track** is driven by [`Code: RUG Orchestrator`](.github/agents/code-rug-orchestrator.agent.md) using the **Repeat Until Good (RUG)** protocol:

- **Pure Orchestration**: The lead agent **never** writes code, edits files, or executes commands directly. It delegates 100% of execution to specialized subagents.
- **Independent Validation**: Every work subagent's changes are verified by a separate validation subagent with a fresh context window.
- **Persistent Run & Decision Records**: Every completed implementation session compiles a durable log in `docs/implementation/RUN-YYYYMMDD-[slug].md` capturing technical trade-offs, deviations from spec, and empirical test evidence.
- **Reverse Escalation Protocol (Rolling Back Up)**: If an upstream specification is technically impossible or contradicts domain invariants, the orchestrator halts execution, formulates a Spec Change Proposal (SCP), and yields to a human decision gate rather than looping indefinitely on code retries.
- **Strict In-Domain Workers**: Only code implementation, debugging, refactoring, and code verification subagents are dispatched.

### RUG Execution Loop & Subagent Hierarchy

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
    'edgeLabelBackground': '#1a1d27'
  }
}}%%
flowchart TD
    classDef primary fill:#422026,stroke:#e06c75,stroke-width:1.5px,color:#fde8ec;
    classDef secondary fill:#1b3528,stroke:#73c991,stroke-width:1.5px,color:#e6f7ee;
    classDef tertiary fill:#1d2c44,stroke:#61afef,stroke-width:1.5px,color:#e4f0fc;
    classDef note fill:#2e271a,stroke:#e5c07b,stroke-width:1.5px,color:#fdf4db;

    REQ["📋 Task Request / Approved Requirements"]:::tertiary
    RUG["🎯 Code: RUG Orchestrator<br/><i>(Pure Coordinator: Never edits or runs tests directly)</i>"]:::primary
    TODO["📝 Todo List Roadmap<br/><i>(manage_todo_list)</i>"]:::note

    subgraph WORKERS["Implementation Workers (In-Domain Subagents)"]
        SWE["💻 Code: SWE<br/><i>(Mandatory Worker: Features, edits, refactoring, tests)</i>"]:::secondary
        DEBUG["🔍 Code: Debug<br/><i>(Optional / Conditional: 4-phase bug diagnosis & reproduction)</i>"]:::secondary
    end

    subgraph VALIDATION["Validation Subagents (Independent Context)"]
        QA_LITE["⚡ Code: QA Lite<br/><i>(Default Task Validator: Fast diff inspection & sanity check)</i>"]:::secondary
        QA_FULL["🧪 Code: QA<br/><i>(Strict Mode / High-Risk: Deep test suite execution & regressions)</i>"]:::secondary
        SEC["🛡️ Code: Security Reviewer<br/><i>(Optional / Triggered: OWASP, Zero Trust, LLM threats)</i>"]:::secondary
    end

    REQ --> RUG
    RUG -->|"1. Decompose & Initialize"| TODO

    RUG -->|"Standard Task Dispatch"| SWE
    RUG -.->|"Defect Diagnosis / Fix"| DEBUG

    SWE & DEBUG -->|"Produces Diff & Self-Report"| MODE_SELECT{"Validation Mode"}:::primary

    MODE_SELECT -->|"Default / Fast Mode"| QA_LITE
    MODE_SELECT -->|"High-Risk Task / Strict Mode"| QA_FULL
    MODE_SELECT -.->|"Security Surface (Optional)"| SEC

    QA_LITE & QA_FULL & SEC --> VERDICT{"Validation Outcome"}:::primary

    VERDICT -->|"FAIL (Code Defect)"| RETRY["🔁 Re-dispatch Worker with Defect Report"]:::primary
    RETRY --> SWE

    VERDICT -->|"FAIL (Spec Defect / Blocker)"| ESCALATE["⚠️ Escalate: Spec Change Proposal (SCP)"]:::note
    ESCALATE --> USER_GATE["🔒 User Decision Gate: Roll back to Spec Track or Authorize Exception"]:::note
    USER_GATE -.->|"Roll Back"| REQ
    USER_GATE -.->|"Exception Approved"| SWE

    VERDICT -->|"PASS"| ADVANCE["✅ Mark Task Complete in Todo"]:::note
    ADVANCE --> MORE_TASKS{"More Tasks in Todo?"}:::primary

    MORE_TASKS -->|"Yes: Next Task"| RUG
    MORE_TASKS -->|"No: All Tasks Done"| FINAL_GATE["🏁 Final Integration Gate<br/><i>(Code: QA full test suite run)</i>"]:::primary
    FINAL_GATE --> LOG["📝 Compile Run & Decision Log<br/><i>(docs/implementation/RUN-*.md)</i>"]:::tertiary
    LOG --> COMPLETE["🎉 Verified Working Solution"]:::tertiary
```

### Validation Modes & Flags

The RUG Orchestrator controls automated verification rigor via mode flags:

- **Default (Balanced)**: Dispatches `Code: QA Lite` for per-task static checks. Tasks modifying core invariants, security/auth, or public schemas escalate to full `Code: QA`. Runs full `Code: QA` at the final integration gate.
- **Fast / Draft Mode (`--fast`, `--draft`)**: Uses `Code: QA Lite` for task-level checks and the final integration gate, maximizing iteration velocity.
- **Strict / Release Mode (`--strict`, `--release`)**: Dispatches full `Code: QA` for **every** individual task and the final gate, executing complete test suites (`cargo test`, Python unittests) and linter passes.

### `Code:*` Subagents Catalog

| Agent Name | Role | Call Nature | Trigger Condition | Primary Deliverables |
| :--- | :--- | :--- | :--- | :--- |
| **`Code: RUG Orchestrator`** | Execution Coordinator | **Orchestrator** | Feature request, task spec | Task decomposition, subagent prompts, iteration loop, run logs |
| **`Code: SWE`** | Senior Software Engineer | **Mandatory Worker** | Standard implementation task | Clean code diffs, unit/integration tests, docstrings, technical decisions |
| **`Code: Debug`** | Defect Diagnostician | **Optional / Conditional** | Bug report, failing tests | Reproduction recipe, root cause analysis, targeted fix |
| **`Code: QA Lite`** | Fast Sanity Validator | **Default Task Auditor** | Routine task completion | Diff sanity verification, acceptance criteria checklist, defect classification |
| **`Code: QA`** | Comprehensive QA Tester | **Strict / Integration Auditor** | High-risk tasks, final integration | Test suite logs (`cargo test`, `unittest`), boundary audit, regression proofs |
| **`Code: Security Reviewer`** | Code Security Specialist | **Optional / Triggered** | Auth, crypto, external boundaries | OWASP Top 10 audit, Zero Trust verification, risk report |

---

## 🔬 Scientific Research Track (`Sci:*`)

The **Scientific Track** coordinates empirical discovery campaigns. Managed by [`Sci: Orchestrator`](.github/agents/sci-orchestrator.agent.md)—acting as a Lead Scientist that drives goal-oriented empirical discovery, creative ideation, and rapid falsification—it coordinates 2 substantive subagents to enforce a disciplined research lifecycle:

- **Theoretical & Measurement Purity**: Separates mathematical modeling and empirical experiment protocol design (`Sci: Theory & Protocol`) from experiment execution and empirical diagnostics (`Sci: Execution & Analysis`).
- **Direct Execution Dispatch (1-Level Architecture)**: To eliminate illegal nested-subagent calls (`Sci` $\rightarrow$ `RUG` $\rightarrow$ `SWE`) and avoid burdensome enterprise release ceremonies for exploratory script adjustments, `Sci: Orchestrator` directly dispatches `Sci: Execution & Analysis` within its own 1-level hierarchy.
- **Two-Tier Loop Architecture (Inner vs. Outer Loop)**: Decouples autonomous parameter exploration from macro-level scientific evolution. The inner loop searches parameter regimes intelligently, while the outer loop drives algorithmic mutations, structural ablations, and complexity progression.
- **Language-Scoped Experiment Isolation**: Every experiment is provisioned as an independent, immutable package under the respective language tree adhering to language identifier rules (lowercase with underscores, e.g. `python/experiments/exp_yyyy_nnna_[slug]/`), strictly preserving historical reproducibility.
- **Persistent Campaign Continuity & Provenance**: Tracks theoretical capability progression on a complexity ladder across sessions in `docs/research/CAMPAIGN.md`, with every run pinned by cryptographic checksums and Git tags.
- **Programmatic Telemetry Reduction**: Raw multi-gigabyte telemetry is reduced via scripts to statistical summaries and phase plots before ingestion for analysis.
- **Goal-Driven Autonomous Discovery Loop**: Navigates towards ultimate research milestones via iterative hypothesis formulation, fast falsification (the two-strike rule), and mechanistic pivots under an asset-enforced 5-cycle budget in `CAMPAIGN.md`. Decision gates (**Gate H/P** and **Gate I**) execute autonomously unless tripped by genuine multi-path ambiguity, a 2-strike paradigm stall, or budget exhaustion.
- **Autonomous & Terminal Research Scope**: The research track is self-contained. Its mission terminates not in production software releases, but in rigorous empirical discovery, reproducible experiment packages, execution manifests, and durable diagnostic dossiers.

### Research Lifecycle & Execution Handoff

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
    'edgeLabelBackground': '#1a1d27'
  }
}}%%
flowchart TD
    classDef primary fill:#422026,stroke:#e06c75,stroke-width:1.5px,color:#fde8ec;
    classDef secondary fill:#1b3528,stroke:#73c991,stroke-width:1.5px,color:#e6f7ee;
    classDef tertiary fill:#1d2c44,stroke:#61afef,stroke-width:1.5px,color:#e4f0fc;
    classDef note fill:#2e271a,stroke:#e5c07b,stroke-width:1.5px,color:#fdf4db;

    subgraph FORMULATION["1. Scientific Formulation (In-Domain Sci Subagents)"]
        THEORY["📐 Sci: Theory & Protocol<br/><i>(Hypotheses & Protocols)</i>"]:::secondary
    end

    subgraph PROTOCOL_GATE["2. Protocol & Budget Check Gate"]
        GATE_HP["⚡ Gate H/P: Protocol & Budget Check<br/><i>(Autonomous pre-check; escalates if ambiguous)</i>"]:::note
    end

    subgraph EXECUTION["3. Execution & Provenance Capture (Direct 1-Level Dispatch)"]
        EXEC["💻 Sci: Execution & Analysis<br/><i>(Provisions, runs, reduces, diagnoses)</i>"]:::secondary
        EXP_PKG["📦 Isolated Experiment Package<br/><i>(python/experiments/exp_*/ )</i>"]:::tertiary
        RUN_MANIFEST["📋 Run Manifest & Telemetry<br/><i>(RUN-EXP-*.md & data/telemetry/)</i>"]:::tertiary
        EXEC -->|"Provisions & runs"| EXP_PKG
        EXP_PKG -->|"Emits Raw Logs & Manifest"| RUN_MANIFEST
        RUN_MANIFEST -.->|"Reduced & Analyzed"| EXEC
    end

    subgraph EVALUATION["4. Empirical Diagnostics & Discovery Loop"]
        ORCH_ITER["🎯 Sci: Orchestrator<br/><i>(Strategic Assessment & Iteration Decision)</i>"]:::primary
        GATE_I["⚡ Gate I: Iteration Check Gate<br/><i>(Autonomous loop; escalates on stall or 5-cycle limit)</i>"]:::note
    end

    ORCH_ITER -->|"Directs formulation"| THEORY
    THEORY -->|"Protocol & Implementation Spec"| GATE_HP
    GATE_HP -->|"Approved"| EXEC
    EXEC -->|"Diagnostic Evaluation Report"| ORCH_ITER
    ORCH_ITER -->|"Iteration Directive"| GATE_I
    GATE_I -->|"Mutate / Advance / Ablate / Pivot"| THEORY
    GATE_I -.->|"Milestone Verified"| MILESTONE_DONE["🏁 Research Milestone Complete<br/><i>(Validated Research Dossier & Pinned Tags)</i>"]:::note
```

### 🏛️ Scientific Version Control, Provenance & Reproducibility Architecture

#### The Philosophy: Good Science Demands Frozen History

Scientific inquiry is inherently evolutionary, non-linear, and branching. In conventional software engineering, code changes converge linearly toward a release, and older implementations are continuously refactored or overwritten. In scientific research, however, mutating code in place is disastrous: it silently invalidates prior baselines, making retrospective auditing, peer review, and comparative ablation impossible.

Once an experiment is executed and its manifest is published, that experiment's code, configurations, seeds, and analytical interpretations must remain **permanently frozen, auditable, and runnable**.

#### The Separation of "Apparatus" vs. "Experiment Packages"

To maintain clarity and prevent code rot, the codebase strictly separates shared scientific infrastructure from individual experiment instances:

1. **The Shared Lab Apparatus (`python/src/tools/` or Rust `src/`)**:
   - Reusable simulation engines, dynamical operators, linear algebra utilities, telemetry loggers, reduction tools, and phase-space projection calculators.
   - Tested under [`python/tests/`](python/tests/) and maintained with strict backward-compatibility so earlier experiments never break when shared tools improve.
2. **Isolated Experiment Packages (`python/experiments/exp_yyyy_nnna_[slug]/`)**:
   - Every experiment is provisioned as an independent package under its language directory containing its specific algorithmic implementation (`dynamics.py`), inner-loop sweep configurations (`config.toml`), runner entrypoint (`run.py`), and localized test/verification routines. All directory and module names use lowercase with underscores (`snake_case`) for valid language import syntax.
   - When an experiment is completed, this directory becomes **strictly read-only**.
   - If cycle 2 requires an algorithmic mutation, it creates a new package (`python/experiments/exp_yyyy_nnnb_[slug]/`) that can import from `tools` or subclass prior experiments, clearly declaring its parent lineage in its metadata.

#### Two-Tier Loop Architecture: Inner vs. Outer Loop

The multi-agent research track formally distinguishes between exploratory parameter discovery and structural scientific evolution:

- **The Inner Loop (Autonomous Parameter & Regime Discovery)**:
  - Governed by `Sci: Execution & Analysis` within an isolated experiment package.
  - Rather than executing a rigid or blind brute-force grid, the worker leverages LLM intelligence to adaptively probe the parameter space, discover regime transitions, observe phase portraits, and home in on critical boundaries (e.g. attractor stability vs. chaotic collapse).
  - The trajectory of parameter exploration is recorded directly in the run manifest without requiring multi-agent committee deliberation for every hyperparameter adjustment.
- **The Outer Loop (Scientific Evolution & Complexity Ladder)**:
  - Governed by the Science Orchestrator and its 2 subagents ([`Sci: Theory & Protocol`](.github/agents/sci-theory-protocol.agent.md) and [`Sci: Execution & Analysis`](.github/agents/sci-execution-analysis.agent.md)).
  - Operates across the macro-level scientific state machine: formulating new mathematical hypotheses, designing novel experiment packages, diagnosing complex dynamical behaviors, and executing structural mutations (`MUTATE`), mechanism ablations (`ABLATE`), or complexity ladder advancements (`ADVANCE`).

#### Git Hygiene & Tagging: Why Single-Branch Tagging Wins

Rather than fragmenting the repository into dozens of divergent Git branches, the scientific track operates on **single-branch additive directories coupled with Git tags** (`exp/EXP-YYYY-NNNa-[run-id]`):

- **LLM Tool Ergonomics**: LLM agents inspect and diff code using `read`, `grep_search`, and file inspection tools. If past experiments live on separate git branches, agents cannot inspect or compare algorithms without executing git checkout commands—which alters the working tree, invalidates caches, and risks uncommitted file loss. In additive directories, past experiments are immediately visible side-by-side.
- **IDE & Language Server Stability**: Switching Git branches causes filesystem churn, reloads file watchers, and forces Rust Analyzer and Python language servers (Pyright/mypy) to flush caches and re-index. Single-branch additive layouts maintain complete IDE stability.
- **Cryptographic Immutability**: Applying a Git tag upon run completion pins the exact commit hash of the code, configs, protocol, and run manifest at that point in time.
- **Clean Working Tree Invariant**: Sweeps are never executed or logged with uncommitted changes. `Git Status Dirty` must be `No` in every official run manifest.

### Science-to-Engineering Protocol & Execution Gate

The `Sci: Theory & Protocol` subagent includes an **Experiment Implementation Specification** in every protocol document before presenting it at **Gate H/P**:

1. **Target Experiment Package**: Exact package path under the language tree adhering to Python/Rust identifier rules: lowercase with underscores (e.g. `python/experiments/exp_yyyy_nnna_[slug]/`).
2. **Parent Lineage**: Explicit parent protocol reference (if mutating or ablating a prior experiment).
3. **CLI Entry Points**: Exact executable commands, script targets, and argument signatures.
4. **Parameter Search Space & Strategy**: Explicit parameter boundaries, seed sets, and guidance for adaptive discovery within the space.
5. **Emission Schemas**: JSON/CSV telemetry fields, logging frequencies, and target directories under `data/telemetry/`.
6. **Resource Budgets**: Wall-clock timeouts, memory limits, and process limits.
7. **Success Gates & Reduction**: Minimum metric thresholds and automated telemetry reduction targets before returning logs for analysis.

Once signed off at **Gate H/P**, `Sci: Orchestrator` directly dispatches `Sci: Execution & Analysis` to provision the experiment package, execute the adaptive sweep, execute telemetry reduction (`python/scripts/reduce_telemetry.py`), commit the clean code, create the Git tag, and log the `RUN-EXP-*.md` manifest.

### `Sci:*` Subagents Catalog

| Agent Name | Lifecycle Stage | Role | Primary Input | Primary Output |
| :--- | :--- | :--- | :--- | :--- |
| **`Sci: Orchestrator`** | Pipeline Controller & Lead Scientist | Strategic direction, iteration decisions, state machine management, work package assembly | Research goal, campaign state, diagnostic reports | Strategic directives, iteration directives, dispatches, campaign state tracking, Git tag governance |
| **`Sci: Theory & Protocol`** | Theoretical Modeling & Empirical Design | Mathematical formalizer and measurement architect | Work package with strategic directive and inline context | Formal Hypothesis Documents (`HYP-*.md`) + Structured Experiment Protocols with Implementation Specs (`EXP-*.md`) |
| **`Sci: Execution & Analysis`** | Execution, Telemetry & Diagnostics | Experiment runner and empirical diagnostician | Work package with approved protocol and implementation spec | Isolated experiment packages, telemetry, Run Manifests (`RUN-EXP-*.md`), Diagnostic Evaluation Reports (`DIAG-*.md`), Git tags |

---

## 🔄 Inter-Track Artifact Handoff Matrix

Because agents operate under strict domain encapsulation and a 1-level depth limit, cross-track coordination occurs through **asynchronous artifact handoffs mediated by the Operator**:

| Source Track | Originating Artifact | User Handoff Action | Destination Track | Target Orchestrator | Expected Outcome |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`Spec:*`** | Approved Requirements (`docs/requirements/`) & ADRs (`docs/architecture/`) | Operator triggers implementation | **`Code:*`** | `Code: RUG Orchestrator` | Fully implemented, unit-tested, and verified software |
| **`Sci:*`** | Validated Research Findings & Manifests (`docs/research/CAMPAIGN.md`, `DIAG-*.md`, `RUN-EXP-*.md`) | Operator reviews scientific conclusion and tags | **Operator / Records** | N/A | Durable, reproducible scientific dossier with pinned code tags and telemetry manifests |
| **`Code:*`** | Run Manifest (`docs/research/runs/RUN-EXP-*.md`) & Reduced Telemetry (`data/telemetry/`) | Operator passes verified data to research pipeline (if runner used externally) | **`Sci:*`** | `Sci: Orchestrator` | Empirical diagnostics and adaptive iteration directives |
| **`Code:*`** | Spec Change Proposal (Escalation report on infeasible requirement) | Operator routes architectural roadblock/defect | **`Spec:*`** | `Spec: Orchestrator` | Updated formal requirements (`REQ-T*`) or new ADR (`ADR-NNNN-*`) |
| **`Code:*`** | Run & Decision Log (`docs/implementation/RUN-YYYYMMDD-*.md`) | Operator reviews implementation decisions & test proof | **Operator / Records** | N/A | Durable audit trail of plan vs implementation drift and decisions |
| **`Sci:*`** | Research Campaign State Tracker (`docs/research/CAMPAIGN.md`) | Operator monitors complexity ladder progression | **Operator / Records** | N/A | Cross-session continuity of milestones, hypotheses, and iteration history |

---

## 🎨 Diagramming Style Guide

All architectural and workflow diagrams in documentation must use Mermaid and adhere to the project's **Gentle RGB Dark Theme**:

- `:::primary` (Rosewood Red `#422026` / `#e06c75`): Core domain, invariants, decision gates, entry points, orchestrators.
- `:::secondary` (Forest Sage Green `#1b3528` / `#73c991`): Application services, coordinators, worker subagents, active pipelines.
- `:::tertiary` (Royal Slate Blue `#1d2c44` / `#61afef`): Storage, databases, adapters, external boundaries, specifications and artifacts.
- `:::note` (Muted Amber `#2e271a` / `#e5c07b`): Security constraints, verification guards, stage sign-off gates, callouts.

See [`docs/templates/mermaid-style-guide.md`](docs/templates/mermaid-style-guide.md) and [`docs/templates/diagram-template.md`](docs/templates/diagram-template.md) for full style definitions, class declarations, and copy-pasteable boilerplates.

---

## 📁 Project Structure & Layout

This template enforces a strict separation between Rust high-performance core logic and Python tooling:

```text
.
├── .agents/agents/            # Agent definition copies for runtime discovery
├── .github/agents/            # GitHub Copilot agent track definitions (spec-*, code-*, sci-*)
├── data/                      # Experiment runtime data
│   └── telemetry/             # Raw experiment telemetry and reduced summaries
├── docs/                      # Architecture, ADRs, requirements & templates
│   ├── architecture/          # ADRs (adr-*.md) and ADR registry (README.md)
│   ├── implementation/        # Implementation Run & Decision Logs (RUN-YYYYMMDD-*)
│   ├── requirements/          # Formal r9ts requirements & audits
│   │   ├── product/           # REQ-T0-* domain invariants
│   │   ├── architecture/      # REQ-T1-* logical architecture contracts
│   │   ├── system/            # REQ-T2-* technology realization profiles
│   │   ├── audits/            # AUDIT-T*-* tier audit reports
│   │   └── ROADMAP.md         # Persistent specification state tracker
│   ├── research/              # Scientific research campaign artifacts
│   │   ├── hypotheses/        # Formal hypotheses (HYP-YYYY-NNN.md)
│   │   ├── protocols/         # Experiment protocols & specs (EXP-YYYY-NNNa.md)
│   │   ├── runs/              # Execution manifests & run logs (RUN-EXP-*.md)
│   │   ├── diagnostics/       # Diagnostic evaluation reports (DIAG-*.md)
│   │   └── CAMPAIGN.md        # Persistent campaign state tracker
│   └── templates/             # Templates for ADRs, requirements, roadmaps, and runs
├── python/                    # Python workspace
│   ├── pyproject.toml         # Python dependency definitions
│   ├── experiments/           # Additive, immutable experiment packages (exp_yyyy_nnna_*)
│   ├── scripts/               # One-off utility scripts
│   ├── src/tools/             # Reusable Python tools / lab apparatus (imported as `tools`)
│   └── tests/                 # Python unit and integration tests
├── src/                       # Rust primary workspace (main.rs, core libraries)
├── tests/                     # Rust integration tests
├── Cargo.toml                 # Rust workspace package configuration
├── GEMINI.md                  # Project rules & developer guidelines
└── README.md                  # Project documentation & agent orchestration guide
```

### Dual-Language Rules

- **Rust**: Put production code in `src/` and integration tests in `tests/`. Do not mix Python into `src/`.
- **Python**: Use `.venv/bin/python`. Declare dependencies in `python/pyproject.toml`. Reusable Python packages reside in `python/src/tools/` and are imported as `tools`. Isolated experiment packages reside in `python/experiments/` using lowercase `snake_case` package names (e.g. `exp_yyyy_nnna_[slug]`). Do not modify `PYTHONPATH`.

---

## 🧪 Validation & Quality Assurance

Always run validation checks on affected surfaces before committing changes:

### Rust Validation

```bash
cargo fmt --check
cargo clippy
cargo test
```

### Python Validation

```bash
.venv/bin/python -m unittest discover -s python/tests -v
```

### Markdown & Diagram Validation

```bash
# Automatically fix formatting issues
markdownlint-cli2 --fix "**/*.md"

# Verify clean markdown compliance
markdownlint-cli2 "**/*.md"
```

---

## 📄 License

Distributed under the [MIT License](LICENSE).
