# Adaptive Computational Substrate

## The Problem

Modern deep learning is powerful but structurally rigid:

- **Train once, freeze forever.** A model is optimised offline over a static dataset, then deployed as a fixed function. There is no mechanism for continuous adaptation to a changing world without retraining from scratch or fine-tuning that erases prior knowledge (catastrophic forgetting).
- **Recompute everything, every time.** A transformer processes every token in the context window on every forward pass. There is no persistent state between calls — the entire "memory" is reconstructed from the KV cache on each invocation, and that cache is volatile.
- **Brute-force attention.** Global all-to-all attention gives every token direct access to every other token. This is computationally convenient but scales quadratically, has no notion of locality, and bears no resemblance to how any physical or biological system processes information.
- **Energy-oblivious.** Dense floating-point tensor contractions on GPUs operate orders of magnitude above the thermodynamic minimum. The human brain runs on ~20W; a single GPU training run can consume megawatt-hours.

These are not incidental costs. They are structural consequences of the architecture: a monolithic parameter tensor, optimised by global gradient descent, with no local state, no temporal dynamics, and no self-repair.

## The North Star

> **Engineer an adaptive computational substrate where inference, learning, and structural reorganisation are a single continuous process — running efficiently on standard CPU/GPU hardware — and where the system's behaviour emerges from local temporal dynamics rather than global optimisation.**

This is not a call to abandon deep learning. It is a call to explore whether there exists a qualitatively different computational paradigm that:

1. **Learns continuously** — new information is incorporated without destroying old knowledge, without replay buffers, and without retraining.
2. **Maintains persistent state** — memory is a physical configuration of the system, not a volatile cache reconstructed on every inference.
3. **Operates at the edge of chaos** — the system's dynamics are neither rigidly deterministic (like transformer feedforward passes) nor purely random, but occupy the critical regime where complex, adaptive behaviour emerges.
4. **Runs on real hardware** — the target platform is standard CPU and GPU silicon, not hypothetical neuromorphic chips. Efficiency is measured against what this hardware actually does well.

## Stable Goals

These are the properties we are trying to achieve. They do not prescribe any particular mechanism.

### G1 — Continuous adaptation without catastrophic forgetting

The system incorporates new sequential information into persistent internal state without overwriting or degrading previously acquired competencies.

**Measured by:** Backward Transfer (BWT) $\ge 0$ across a sequence of tasks presented over time:

$$\text{BWT} = \frac{1}{T-1}\sum_{i=1}^{T-1}(R_{T,i} - R_{i,i})$$

where $R_{T,i}$ is performance on task $i$ after learning tasks $1 \ldots T$.

### G2 — Persistent, bounded memory

Information, once encoded, persists in the system's state without continuous re-stimulation. Memory does not grow linearly with sequence length — it is bounded by the system's physical capacity, not by the input history.

**Measured by:** State retention fidelity over time, and the relationship between system size and memory capacity.

### G3 — Local temporal dynamics

Computation arises from the interaction of locally connected elements evolving over time. No element has global access to the full system state. Information propagates through the system at finite speed, mediated by local interactions.

**Measured by:** Bounded connectivity per element, and the system's ability to perform non-trivial computation despite strict locality constraints.

### G4 — Efficiency on standard hardware

The system runs on CPUs and GPUs. It exploits the hardware's actual strengths: cache locality, SIMD vectorisation, parallel thread execution, and cheap digital memory. Operations that are trivially efficient on digital hardware (storing a value, looking up an address) should remain trivially efficient in the substrate — not be re-invented from more expensive primitives.

**Measured by:** Wall-clock time and energy per operation, compared to conventional implementations of the same capability on the same hardware.

### G5 — Self-organisation and homeostasis

The system regulates its own internal dynamics without centralised control. It avoids pathological regimes (extinction, saturation) through local feedback mechanisms. Its operating point is maintained near criticality — the boundary between order and chaos where computational capacity is maximised.

**Measured by:** Firing density / activity metrics remaining in a bounded operating band without external tuning, across a range of input conditions.

### G6 — Emergent computational capability

The system's ability to perform complex tasks (sequence prediction, pattern completion, associative retrieval) is not hard-wired — it emerges from the collective dynamics of simple local elements. New capabilities arise from scaling, restructuring, or recombining existing components, not from designing bespoke circuits for each task.

**Measured by:** The challenge milestones below.

## Hardware Constraint

The target platform is **modern commodity CPU and GPU hardware**. This is a hard constraint, not an aspiration:

- **Digital memory is cheap.** Storing a value in a register, a cache line, or a GPU shared memory cell costs effectively nothing. Any substrate design that re-implements basic storage from more expensive dynamical primitives (e.g., 272 spiking nodes to hold 16 bits) is failing at G4, regardless of how scientifically interesting it might be.
- **SIMD and parallelism are the efficiency lever.** CPUs have AVX-512; GPUs have thousands of cores executing the same instruction. Designs that can be expressed as uniform parallel operations over arrays will run orders of magnitude faster than designs requiring irregular graph traversals with unpredictable branching.
- **Cache locality matters.** Sequential memory access patterns are 100× faster than random access on modern hardware. Spatial locality in the substrate's state representation translates directly to performance.
- **The baseline for comparison is conventional software.** If a transformer with 1M parameters can do next-token prediction at X tokens/second on a given GPU, the substrate must be compared against that benchmark, not against a hypothetical neuromorphic chip.

This does not mean the substrate must look like a transformer. It means the substrate must *run efficiently on the same hardware* that transformers run on. The innovation is in the computational model, not in the hardware.

## Approach: Open and Exploratory

Unlike the goals, the approach is explicitly **not fixed**. The following are directions worth exploring, not commitments:

### Temporal spiking and edge-of-chaos dynamics

Binary spike-based communication — where information is encoded in the timing and pattern of discrete events rather than continuous floating-point values — has appealing properties:

- **Energy proportionality.** Silence is free; only activity costs compute. This is the opposite of dense tensor contractions where every element participates on every step.
- **Natural temporal encoding.** Time is a first-class citizen, not something bolted on via positional embeddings.
- **Edge-of-chaos dynamics.** Spiking networks with local homeostatic feedback can self-organise to the critical regime where computational capacity (separation property, memory capacity) is maximised. This is qualitatively different from the rigid deterministic computation of transformer feedforward passes.

### Reservoir computing

The reservoir computing paradigm — a fixed or slowly adapting recurrent dynamical system with a trainable readout layer — separates the "rich dynamics" problem from the "learning" problem. The reservoir provides a high-dimensional temporal embedding of the input; the readout is a simple linear map. This is potentially a good fit for the substrate because:

- The reservoir's dynamics provide the temporal memory and non-linear mixing (G1, G2, G3).
- The readout provides a conventional optimisation target that can be trained efficiently (G4).
- The reservoir can be implemented as a parallel array operation on GPU (G4).

### Cellular automata on GPU grids

Dense 1D or 2D cellular automata, operating on arrays that map directly to GPU memory layout, combine locality (G3) with hardware efficiency (G4). Rules can be expressed as lookup tables or small neural networks applied uniformly across the grid — ideal for SIMD execution. The challenge is whether such systems can develop sufficient computational depth.

### Hybrid architectures

The substrate need not be pure. A design might combine:

- A **dynamical core** that provides temporal state, self-organisation, and edge-of-chaos behaviour
- **Conventional digital memory** for efficient storage (respecting G4)
- A **trainable interface** (readout/projection layers) that translates between the substrate's internal dynamics and external task requirements

The only prohibition is on approaches that are *structurally incapable* of achieving the goals — e.g., any system that requires global gradient descent through the full state for learning (violates G1 and G3), or any system that requires bespoke hardware to run efficiently (violates G4).

---

## Challenge Milestones

These are **problems that any successful substrate must solve**. They are ordered roughly by complexity, but they are not a strict sequential ladder — a substrate might tackle them in a different order, or solve multiple simultaneously, depending on its architecture.

The milestones are framed as **capabilities**, not implementations. They say *what* the system must do, never *how*.

> [!IMPORTANT]
> Each milestone defines a **black-box interface contract**: `encode_input → run_dynamics → decode_output → metric`. The substrate's internal mechanism is unconstrained. Any approach that meets the metric on held-out evaluation is valid.

### C1 — Non-linear temporal mixing

> Can the system perform a computation that requires both non-linear combination of inputs and memory of past inputs?

**Task:** Delayed XOR — predict $y_t = x_t \oplus x_{t-\tau}$ for $\tau \ge 5$ time steps, where $x_t \in \lbrace 0, 1 \rbrace$.

**Why this matters:** A memoryless system cannot solve this (it requires remembering $x_{t-\tau}$). A linear system cannot solve this (XOR is non-linear). Any substrate that passes this gate has both memory and non-linearity — the two minimal ingredients for general computation.

**Success metric:** Accuracy $\ge 95\%$ on held-out sequences with $\tau \ge 5$.

### C2 — Persistent state retention

> Can the system store a value and retain it indefinitely without continuous re-stimulation?

**Task:** Write a binary value via an input signal, then remove all input. Read the value back after $\Delta t \ge 10^3$ time steps of autonomous evolution.

**Why this matters:** Sequence modelling requires working memory. If state decays, the system can only process local windows, not long-range dependencies.

**Success metric:** Read accuracy $= 100\%$ after $10^3$ autonomous steps.

### C3 — Sequential state tracking

> Can the system maintain and update internal state based on a streaming input sequence?

**Task:** Recognise a regular language — e.g., track the parity (even/odd count) of a streaming bitstream, or recognise the pattern `(10)+1`.

**Why this matters:** This is the minimum requirement for any system that processes sequences: it must track state that depends on the entire history, not just the most recent input.

**Success metric:** Zero classification error on sequences of length $L \in [10, 100]$.

### C4 — Hierarchical structure (context-free)

> Can the system handle nested, recursive structure — not just flat sequential patterns?

**Task:** Recognise Dyck languages — balanced parentheses `(()())` and nested multi-bracket patterns `([{}])`.

**Why this matters:** Natural language has recursive structure (nested clauses, long-range agreement). Context-free languages are the simplest class that captures this. A system limited to regular languages (C3) cannot handle it — this requires stack-like or equivalent memory.

**Success metric:** Correct classification on nesting depths $2\times$ deeper than seen during any optimisation or calibration phase.

### C5 — Associative retrieval

> Can the system store multiple key-value bindings and retrieve any value on demand, even after intervening distractor inputs?

**Task:** Present $N \ge 16$ key-value pairs $(K_i, V_i)$ interspersed with distractor tokens over a long sequence ($L \ge 256$ tokens). Then query with $K_j$ and require the system to output $V_j$.

**Why this matters:** This is the substrate analogue of attention / associative memory. A system that can do this has dynamic random access — a capability that transformers achieve via the KV cache but at $O(L)$ memory cost. The challenge is to achieve it with bounded state.

**Success metric:** Retrieval accuracy $\ge 99\%$ across variable context horizons.

### C6 — Distributed representations and noise tolerance

> Can the system encode information in distributed, redundant patterns that degrade gracefully under noise?

**Task:** Map tokens to distributed state configurations. Classify noisy or corrupted inputs by proximity to learned attractor basins.

**Why this matters:** Brittle point-representations (one specific bit pattern = one meaning) don't scale. Distributed codes provide error correction, generalisation, and the basis for continuous semantic spaces.

**Success metric:** Smooth degradation curve (not catastrophic failure) as noise increases from 5% to 25%.

### C7 — Probabilistic next-token prediction

> Can the system estimate the conditional probability distribution $P(x_t \mid x_{<t})$ over a finite vocabulary?

**Task:** Next-token prediction on a known probabilistic grammar (PCFG or $k$-th order HMM).

**Why this matters:** This is the core task of language modelling. A system that can do this has crossed from discrete logic into statistical inference.

**Success metric:** Prediction perplexity approaching the theoretical Shannon entropy $H(X)$ of the generating grammar.

### C8 — In-context dynamic routing

> Can the system reconfigure its input-output mapping on the fly based on a few demonstration examples — without changing any persistent parameters?

**Task:** Present 3–5 examples of an arbitrary function (e.g., a character substitution cipher), then test on a new input. The system must apply the demonstrated function.

**Why this matters:** This is few-shot learning — the defining capability of modern LLMs. The system must act as a "programmable substrate" whose behaviour is steered by context.

**Success metric:** Accuracy on the conditioned task significantly exceeding chance ($p < 0.001$).

### C9 — Autoregressive language generation

> Can the system generate coherent natural language text?

**Task:** Autoregressive generation on a small-vocabulary corpus (TinyStories, BabyLM 10M).

**Why this matters:** This is the ultimate integration test. It requires all prior capabilities working together: memory, sequential state, hierarchical structure, distributed representations, and probabilistic prediction.

**Success metric:** Syntactically coherent output over 128-token windows; validation loss competitive with a baseline transformer of comparable parameter count.

### C10 — Scaling laws

> Does the system improve predictably and smoothly as it is made larger?

**Task:** Measure loss against system scale across multiple sizes.

**Why this matters:** Without scaling laws, a system is a curiosity. With them, it is a paradigm — resources can be allocated rationally.

**Success metric:** A linear relationship on a log-log plot ($\mathcal{L} \propto S^{-\alpha}$) demonstrating smooth power-law scaling.

### Milestone summary

| Challenge | Capability | Computational class |
| :--- | :--- | :--- |
| C1 | Non-linear temporal mixing | Reservoir / echo state |
| C2 | Persistent state retention | Bistable memory |
| C3 | Sequential state tracking | Regular languages (DFA) |
| C4 | Hierarchical structure | Context-free languages (PDA) |
| C5 | Associative retrieval | Content-addressable memory |
| C6 | Distributed representations | Error-correcting codes |
| C7 | Next-token prediction | Statistical sequence modelling |
| C8 | In-context learning | Meta-learning / programmable routing |
| C9 | Language generation | Autoregressive modelling |
| C10 | Scaling laws | Paradigm validation |

> [!NOTE]
> These challenges are labelled C1–C10 rather than "Tier 1–10" deliberately. They are not a strict ladder — a substrate architecture might solve C5 before C4, or tackle C7 and C3 simultaneously. The ordering reflects rough complexity, not a mandatory sequence.
>
> However, some dependencies are real: C7 is very unlikely without C1 and C3. C9 is very unlikely without C7. C10 requires C9. The dependencies are empirical, not procedural.

---

## What We Learned From Phase 1

Experiments 001a through 011a explored the **LIF (Leaky Integrate-and-Fire) spiking substrate** — a specific implementation where the atomic unit was a single LIF neuron with fan-in $\le 4$, communicating via binary spikes over directed weighted edges. That campaign is documented in [`CAMPAIGN.md`](research/CAMPAIGN.md) and the experiment packages in [`src/experiments/`](file:///workspaces/ca-experiment/src/experiments/).

### Findings that carry forward

- **Homeostatic threshold adaptation works.** Local feedback that adjusts firing thresholds based on activity rate reliably keeps the network in the critical regime (G5). This principle applies to any spiking or event-driven substrate, not just LIF.
- **Refractory shielding creates directional information flow.** A post-firing lockout period prevents signals from reflecting backwards through recurrent connections. This is a general dynamical principle, not specific to LIF nodes.
- **Temporal spike coding can carry information.** The timing pattern of binary events encodes information that can be read out by coincidence detection. This supports the edge-of-chaos direction.
- **The capability milestones (C1–C5) are achievable.** The LIF substrate demonstrated non-linear mixing, persistent state, sequential tracking, hierarchical (pushdown) memory, and associative retrieval. These capabilities are real, even if the specific implementation was inefficient.
- **Automated parameter search is essential.** Hand-tuning parameters wastes time and can hide methodological defects. Mechanical search found scientifically important results (the 011a confound) that manual exploration missed. The search harness in [`src/substrate/search/`](file:///workspaces/ca-experiment/src/substrate/search/) carries forward as infrastructure.

### Findings that motivate the reset

- **The LIF substrate is catastrophically inefficient on CPU/GPU.** 272 nodes with irregular graph connectivity to store 16 bits violates G4 absolutely. The hardware does not benefit from this structure — it is fighting the hardware at every level (irregular memory access, unpredictable branching, no vectorisation opportunity, graph topology that defeats cache lines).
- **The capability ladder became a build-on-prior-tier construction.** Each experiment assumed the LIF node as an immutable atom and composed circuits from it. The question shifted from "what's the best way to achieve this capability?" to "can this specific substrate achieve it?" — which is a less interesting and less useful question.
- **The 011a confound revealed that mechanism claims built on hand-tuned parameters are fragile.** See [`understanding-the-011a-confound.md`](misc/understanding-the-011a-confound.md) for the full story.
- **The fan-in 4 constraint was arbitrary.** It came from a 2D planar lattice assumption that was relaxed in [ADR-0001](architecture/adr-0001-substrate-connectivity-topology.md) but continued to shape every subsequent experiment.

### Status of Phase 1

Phase 1 is **complete and archived**. The experiments, hypothesis documents, run manifests, and diagnostic reports remain in the repository as a permanent record. Future work should not extend the LIF substrate or build new circuits from LIF nodes unless there is a specific scientific reason to revisit them.

---

## Guidance for Agents

### What has changed

- The LIF node is **not** the atomic building block of this project. It was one exploratory implementation.
- The fan-in 4 constraint is **not** a hard requirement. Connectivity should be whatever serves the goals and the hardware.
- The capability ladder is **not** a strict sequential progression. Challenges can be attacked in any order.
- Efficiency on real CPU/GPU hardware (G4) is a **first-class goal**, not a downstream concern.

### What has not changed

- The North Star: adaptive, self-organising computation that unifies inference and learning.
- The challenge milestones (C1–C10): any solution must solve these.
- The scientific methodology: pre-registered hypotheses, controlled experiments, mechanical parameter search, held-out validation.
- The infrastructure: experiment framework, search harness, agent orchestration, documentation templates.

### Operational ground rules

1. **Black-box evaluation.** All milestones evaluate only the interface contract: `encode_input → run_dynamics → decode_output → metric`. Internal mechanism is unconstrained.
2. **Efficiency is a goal, not a constraint.** A substrate that solves C5 but takes 1000× longer than a hash map is not a success — it is a demonstration of *expressiveness*, which Phase 1 already provided. Phase 2 must demonstrate *competitive efficiency* (G4) alongside capability.
3. **Follow the science.** If edge-of-chaos spiking dynamics turn out to be a dead end on real hardware, pivot. If a hybrid approach outperforms a pure dynamical substrate, adopt it. The goals are stable; the approach is not.
4. **Exploit the hardware.** Design state representations that map to contiguous arrays. Design update rules that vectorise. Use digital memory where digital memory is the right answer. Do not re-invent efficient operations from expensive dynamical primitives.
