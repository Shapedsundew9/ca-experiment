---
name: template-agent-design
description: >-
  Design and maintenance policy for this template's instructions, skills, and
  custom agents across Antigravity and GitHub Copilot. Use when adding, tuning,
  reviewing, or debugging workflow roles, delegation boundaries, customization
  placement, cross-platform mirrors, or context-loading behavior.
---

# Template Agent Design

Use this skill for the template's workflow architecture and design intent. Use
the platform-provided `agent-customization` skill alongside it for current file
formats, frontmatter syntax, and product-specific capabilities.

## Design Goals

1. Load only guidance relevant to the current task.
2. Give each workflow one clear coordinator and each agent one bounded role.
3. Preserve useful autonomy without allowing role, scope, or delegation drift.
4. Make phase changes durable through artifacts rather than assumed chat memory.
5. Keep Antigravity and GitHub Copilot behavior equivalent without maintaining
   duplicate canonical policy.
6. Keep template customizations reusable: never encode a consuming project's
   current experiment, product, milestone, mechanism, or temporary workaround.

## Customization Layers

Place a rule at the lowest-cost layer that can own it clearly:

| Layer | Owns | Must not become |
| :--- | :--- | :--- |
| Repository instructions | Short, universal workspace invariants needed on nearly every task | A workflow manual or agent catalog |
| File instructions | Rules limited to a stable file or directory pattern | A substitute for role ownership |
| Skill | On-demand domain knowledge, decision policy, or multi-step procedure shared by roles | Always-on context or copied agent prose |
| Agent definition | Identity, authority, inputs, outputs, role-specific constraints, and tool boundary | A copy of shared lifecycle or repository policy |
| Script or hook | Deterministic checks, formatting, generation, or enforcement | A prose guideline the model must interpret |
| Documentation | Rationale, examples, and operator-facing architecture | Normative policy agents must rediscover |

When a rule appears in more than one layer, choose one normative owner and make
other layers point to it briefly. Repository instructions should normally name a
skill, not restate it.

## Workflow Architecture

### Depth-One Delegation

The supported topology is:

```text
operator -> orchestrator -> leaf agent
```

- Only an orchestrator delegates.
- A leaf never invokes another leaf or an orchestrator.
- Cross-role work returns to the orchestrator as a concise status plus durable
  workspace artifacts.
- A later leaf is a fresh context. Its work package must restate required facts
  and name preserved artifacts; instructions must not assume context resumption.
- Cross-track transitions return to the operator unless one root orchestrator is
  explicitly designed and authorized to own both tracks.

### Track Intent

Preserve distinct coordination styles rather than forcing one workflow onto all
work:

| Track | Intent | Default control model |
| :--- | :--- | :--- |
| Specification | Converge on explicit requirements and architecture | Deliberate refinement with human decision gates |
| Production code | Converge on verified implementation quality | Repeat-until-good implementation and independent QA |
| Scientific research | Explore and falsify while controlling spend | Pre-registered evidence, bounded iterations, and empirical gates |

Do not nest one track's orchestrator inside another. If a track needs a
specialized capability, add a leaf role or route through the operator.

## Agent Boundaries

Every agent definition should answer only these questions:

1. What role does this agent own?
2. What decisions may it make?
3. What inputs and artifacts does it receive?
4. What outputs or blocker report must it return?
5. What must it not do?
6. Which shared skill contains the detailed procedure?

Use the minimum tools required by the role. Orchestrators receive delegation
tools; leaves do not. In GitHub Copilot, orchestration-only leaves should set
`user-invocable: false`. Restrict the orchestrator's allowed-agent list when the
platform supports it.

Prefer explicit status contracts over vague handoff language. A blocker report
should identify the blocked operation, evidence, minimum required capability,
preserved artifacts, and the decision needed from the orchestrator.

## Efficient Boundaries

Avoid both extremes: unrestricted local reinvention and a context switch for
every helper.

- Define bounded local work using observable conditions such as ownership,
  callers, behavior copied, shared APIs touched, repetition, and approximate
  scope.
- Treat numeric limits as review triggers, not targets or substitutes for
  judgment.
- Route material shared-infrastructure work through the owning role.
- Allow small adapters when they preserve momentum and record them for later
  review.
- Define when an optional phase may be skipped and require the coordinator to
  record the reason. A workflow diagram must not imply a mandatory dispatch that
  its detailed policy calls optional.

## Cross-Platform Sources

- Store canonical skills under `.agents/skills/<name>/`.
- Expose the same skill to GitHub Copilot with a relative symbolic link at
  `.github/skills/<name>` when symbolic links are supported.
- Keep platform-specific agent wrappers in `.agents/agents/` and
  `.github/agents/` because their frontmatter, tool names, and invocation controls
  differ.
- Keep mirrored agent semantics aligned, but adapt names, tool declarations,
  skill paths, and invocation flags to each platform.
- If it is practical to use a symbolic link to expose the canonical skill to multiple platforms, do so.
- Never replace platform-specific wrappers with a symlink unless both platforms
  accept exactly the same file format and semantics.

## Change Procedure

1. **Identify evidence**: name the observed failure, drift, wasted context, or
   missing role boundary. Do not redesign from preference alone.
2. **Choose the owner**: use the layer table to select one normative home.
3. **Inspect locally**: read the current owner, affected agent wrappers, nearest
   workflow contract, and applicable platform customization reference.
4. **State a hypothesis**: describe how the smallest policy change should alter
   behavior and one cheap check that could disprove it.
5. **Edit canonically**: change the normative source first. Remove superseded
   copies rather than adding another formulation.
6. **Adapt wrappers**: update only role-specific or platform-specific text in
   affected agents. Create a GitHub skill symlink when applicable.
7. **Validate mechanically**: check frontmatter, links, tool permissions,
   delegation exposure, symlink resolution, Markdown diagnostics, and diff
   integrity.
8. **Review behaviorally**: ask an independent reviewer to find contradictions,
   ambiguous thresholds, illegal delegation, unnecessary context switches, and
   project-specific leakage.

## Validation Checklist

- The skill name matches its directory and its description includes concrete
  invocation terms.
- Always-on instructions remain short and universally applicable.
- Shared policy has one normative owner.
- Orchestrators alone expose delegation tools.
- Leaves return work to their orchestrator and do not contain handoffs to peers.
- GitHub orchestration-only leaves are not user-invocable.
- Work packages are sufficient for a fresh leaf context.
- Optional dispatches have explicit entry and skip criteria.
- Antigravity and GitHub agent wrappers are semantically aligned.
- GitHub skill links resolve to their canonical `.agents/skills/` targets.
- No template customization names a current project artifact or embeds temporary
  domain assumptions.
- Pre-flight checks, editor diagnostics, and `git diff --check` pass.
- Unrelated user changes remain untouched.

## Anti-Patterns

- Adding detailed workflow policy to always-on repository instructions.
- Copying the same policy into every agent to make it feel more forceful.
- Giving leaves delegation tools or relying on leaf-to-leaf handoffs.
- Writing broad prohibitions without an operational boundary or escape path.
- Solving deterministic enforcement with repeated prose instead of a script,
  hook, schema, or test.
- Encoding one project's current research or product goals into template roles.
- Creating a new role when a skill or scoped work package would suffice.
- Creating a new skill for a rule used by only one agent and one stable task.
- Treating fewer files as the goal when it increases loaded context or blurs
  ownership.
- Mermaid charts should not be used to encode project-specific workflow details; they are for user documentation only.
