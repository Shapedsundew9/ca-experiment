# Multi-Scale Self-Organizing Computational Substrate

To transition from the current deep learning paradigm to a self-organizing, multi-scale computational substrate, the research program requires a precise formulation of what failure modes we are escaping, the target computational principles, and the empirical milestones that define progress.

## **The Foundational North Star**

> **The North Star:** *To engineer an autopoietic, thermodynamically bounded computational substrate wherein sequence modeling, persistent continual learning, and hierarchical abstraction emerge as the native physical relaxation of a decentralized, self-repairing dynamical system.*  
In operational terms, this means replacing the **"train once, freeze forever, recompute everything"** lifecycle of digital Transformers with an **adaptive physical medium** that operates at biological levels of energy efficiency (the human brain consumes approximately $20\text{ W}$) by unifying inference, learning, and structural morphogenesis into a single continuous homeostatic process.

## **Core Research Goals: Current Limits vs. Proposed Advances**

| Architectural Dimension | Existing Paradigm (Transformers & Deep Learning) | Multi-Scale Self-Organizing Alternative | Key Scientific Objective |
| :---- | :---- | :---- | :---- |
| **Operational Lifecycle** | Disconnected: Offline gradient descent over static weights, followed by inference-only execution. | Unified: Continuous, local energy-minimization where inference and learning occur simultaneously. | **Eliminate the training/inference divide.** The system continuously updates its organization on streaming data without explicit optimization phases. |
| **Learning & Plasticity** | Ephemeral activation routing (In-Context Learning) or catastrophic forgetting / plasticity loss under continual fine-tuning. | Multi-scale structural plasticity, Hebbian turnover, and Conceptor subspace partitioning. | **Lifelong, non-interfering memory.** Novel sequence patterns carve out new, orthogonal attractor basins without overwriting historical competencies. |
| **Hardware & Energetics** | Synchronous, dense floating-point tensor contractions (GEMM) operating orders of magnitude above the Landauer limit. | Asynchronous, physics-driven relaxation (analog dynamics, bitwise cellular Boolean operations, or thermodynamic compute). | **Orders-of-magnitude reduction in Energy-Delay Product (EDP)** by substituting digital simulation of math with native physical substrate kinetics. |
| **Memory Topology** | Explicit $\mathcal{O}(1)$ attention matrices and volatile Key-Value (KV) caches scaling with sequence length. | Somatic attractor engrams, localized traveling waves (solitons/SLPs), and bioelectric-like setpoints. | **Bounded, topological state retention.** Sequence context is encoded in self-stabilizing spatio-temporal dynamics rather than growing memory buffers. |
| **Representational Structure** | Monolithic parameter tensors optimized in an unconstrained, hyper-dimensional continuous search space. | Hierarchical Assembly Spaces: validated functional motifs (feedback loops, logic gates) cached in an Assembly Pool. | **Exponential search-space bounding.** Search scales via recursive composition of pre-adapted modules (symbiogenesis) rather than random point perturbations. |

## **Target Quantitative Metrics (Success Indicators)**

To evaluate this paradigm rigorously against mainstream AI benchmarks, experiments should measure:

1. **Continual Learning Efficiency (Plasticity vs. Stability)**:

 $$\text{Backward Transfer (BWT)} = \frac{1}{T - 1} \sum_{i=1}^{T - 1} (R_{T,i} - R_{i,i}) \ge 0$$

 Unlike Transformers where $\text{BWT} \ll 0$ due to catastrophic forgetting, this system must maintain $\text{BWT} \approx 0$ or positive transfer without storing raw data replay buffers.

1. **Thermodynamic Arithmetic Intensity**:

 $$\eta_{\text{thermo}} = \frac{\text{Information Bits Consolidated}}{\text{Joules Dissipated}}$$

 Quantifying how closely the physical or simulated substrate approaches the theoretical Landauer bound ($\Delta E \ge k_B T \ln 2$ per bit erasure) during lifelong sequence learning.

1. **Assembly Depth / Modularity Factor**:

 $$\text{Reuse Ratio} = \frac{\sum a_i(\mathcal{M}_{\text{reused}})}{\text{Total System Degrees of Freedom}}$$

 Tracking whether complex computational graphs are formed through the recursive combination of pre-adapted modules rather than unconstrained parameter growth.

Starting with discrete bit-train automata that settle into attractor basins connects directly to the formalisms of **Liquid State Machines (LSM)**, **Cellular Automata Reservoirs (ReCA)**, and **Stochastic Computing**.

The fundamental trap in unguided recurrent discrete networks is the **ergodic boundary**: without careful tuning, open-ended bit-passing topologies almost always collapse into either trivial extinction (all zeros / static fixpoints) or uncorrelated white noise (hyper-chaos). To make this computationally useful, the starting point must be designed around **Self-Organized Criticality (SOC)** at the smallest possible scale.

---

## Getting Started

### Phase 0: The Minimal Viable Automaton (MVA)

Do not start with an unconstrained graph or continuous signals. Constrain the design space to a tiny, deterministic testbed before allowing the coding to evolve open topologies.

#### 1. The Signal Protocol (Bit-Streams)

* Use synchronous, discrete bit-streams where time $t \in \mathbb{N}$.
* A track between node $i$ and node $j$ transmits a single bit $b \in \lbrace 0, 1 \rbrace$ per clock tick.
* Information is encoded in **temporal pulse spacing** (inter-spike intervals) or **bit density** (Bernoulli rate coding). This natively mimics spike-timing-dependent computation and keeps arithmetic bitwise.

#### 2. Node Architecture (The 4-Neighbor Micro-Core)

* **State:** Each node holds an internal register: a small $k$-bit ring buffer (e.g., $k = 4$ to $8$ bits) representing its local temporal history, plus an accumulated charge counter $V \in [0, V_{\text{thresh}}]$.
* **Incoming Tracks:** Exactly 2 to 4 input tracks.
* **Outgoing Tracks:** Exactly 2 to 4 output tracks.
* **Transition Rule:** At each tick:

1. Incoming bits are logically combined (e.g., bitwise parity, majority voting, or bitwise AND/XOR).
2. If the internal accumulator exceeds $V_{\text{thresh}}$, the node fires a `1` downstream and enters a refractory period ($N_{\text{ref}}$ ticks of forced `0`). Otherwise, it outputs `0`.
3. The internal accumulator decays by a fixed leak factor $\lambda$ on idle ticks.

#### 3. Homeostasis (The Anti-Chaos Shield)

To prevent the network from blowing up into seizure-like saturation or freezing into silence:

* Each node tracks its rolling firing rate $\bar{r}$.
* If $\bar{r} > r_{\text{target}}$, dynamically increment $V_{\text{thresh}}$ (make it harder to fire).
* If $\bar{r} < r_{\text{target}}$, decrement $V_{\text{thresh}}$ (make it more excitable).
* *This single local rule forces the entire substrate toward the "edge of chaos" without centralized tuning.*

---

### Reading In, Settling, and Reading Out

```mermaid
%%{init: {
    'theme': 'base',
    'themeVariables': {
        'darkMode': true,
        'background': '#161922',
        'mainBkg': '#1e2230',
        'nodeBorder': '#434c5e',
        'textColor': '#e2e8f0',
        'fontFamily': 'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
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
flowchart LR
        classDef primary fill:#422026,stroke:#e06c75,stroke-width:1.5px,color:#fde8ec;
        classDef secondary fill:#1b3528,stroke:#73c991,stroke-width:1.5px,color:#e6f7ee;
        classDef tertiary fill:#1d2c44,stroke:#61afef,stroke-width:1.5px,color:#e4f0fc;
        classDef note fill:#2e271a,stroke:#e5c07b,stroke-width:1.5px,color:#fdf4db;

        inputX["Bit Stream X"]:::tertiary
        inputY["Bit Stream Y"]:::tertiary

        subgraph substrate["Recurrent Substrate (16-32 Nodes)"]
                direction TB
                node0["Node 0"]:::secondary <--> node1["Node 1"]:::secondary <--> node2["Node 2"]:::secondary
                node3["Node 3"]:::secondary <--> node4["Node 4"]:::secondary <--> node5["Node 5"]:::secondary
                attractor["Attractor Basin"]:::note
                node1 <--> node4
                node1 --> attractor
                node4 --> attractor
        end

        readout["Bitwise Pooling / Linear Classifier"]:::primary

        inputX --> node0
        inputY --> node3
        node2 --> readout
        node5 --> readout
```

* **Read-In (Encoding):** Reserve 1 to 3 "sensory nodes." Clamp their incoming tracks to the input data bit-train for a fixed window $T_{\text{drive}}$ (e.g., 32 ticks).
* **Dynamic Settling (Relaxation):** After $T_{\text{drive}}$, disconnect or clamp the inputs to a neutral carrier pattern (`0000...` or periodic clock `1010...`). Allow the network to run for $T_{\text{relax}}$ ticks.
* **Settling Criterion:** Track Hamming distance between global network state snapshots $S_t$ and $S_{t-\tau}$. A "settled state" is either:
* A point attractor: $H(S_t, S_{t-1}) = 0$.
* A limit cycle: $H(S_t, S_{t-P}) = 0$ for small period $P$.
* A bounded strange attractor: The trajectory stays within a bounded subspace (measured via state variance).

* **Read-Out:** Extract the binary vector of the network state (either the instantaneous state at $T_{\text{relax}}$ or the time-averaged bit frequencies of all nodes over $T_{\text{relax}}$). Feed this state vector into a simple linear mapping (or majority vote logic) to determine the answer.

### The "Hello World" Task: 2-Bit Temporal XOR

Do not give the agents sequence modeling or language modeling tasks yet. The network must first prove it has **non-linear mixing** and **fading memory**.

* **Input:** Feed a bit $x_t \in \lbrace 0, 1 \rbrace$ into the network at irregular intervals.
* **Target:** Predict $y_t = x_t \oplus x_{t-\tau}$, where $\tau$ is a delay of 3 to 10 ticks.
* **Evaluation:** A feedforward network or simple memoryless CA cannot solve this. The substrate *must* sustain the reverberation of $x_{t-\tau}$ in internal feedback loops while simultaneously performing the non-linear XOR separation with $x_t$.

### Experimental Getting Started Staging Roadmap for Agents

| Phase | Scope | Core Question to Validate | Termination Gate |
| --- | --- | --- | --- |
| **Sprint 0** | 16-node 2D Torus / Small-World Graph | Can local homeostatic threshold adjustment keep bit activity within 5%--20% firing density? | Zero state extinctions; zero full-saturation runaways across $10^5$ ticks. |
| **Sprint 1** | Fixed Topology + Attractor Mapping | Does different input history reliably guide the network into distinct, reproducible limit cycles / attractors? | High separation property: $D(S_A, S_B) > 0$ for distinct inputs $A \neq B$. |
| **Sprint 2** | Temporal XOR Task | Can a simple linear readout extract $x_t \oplus x_{t-\tau}$ purely from the settled substrate dynamics? | $> 95$% accuracy on delayed XOR with $\tau \ge 5$ ticks. |
| **Sprint 3** | Local Plasticity Rules | Can edges rewire or adjust delay tracks via a strictly local, bitwise Hebbian/anti-Hebbian rule without backprop? | Network autonomously improves attractor basin depth for recurring inputs. |

---

## Milestone Tiers

To scale an autonomous scientific agent team from a single non-linear logic primitive (an XOR gate) to modern LLM-level computation, the roadmap  avoids prescribing **implementations** (e.g., forcing a specific grid topology, cellular rule family, or emulated silicon architecture). Instead, it defines **capability gates**—functional, verifiable milestones where each level unlocks a new computational class on the path to sequence modeling and reasoning.

```text
[Isolated Gate] ──► [Cascaded Logic] ──► [Persistent State] ──► [Hierarchical Memory] ──► [Dynamic Routing] ──► [In-Context Learning] ──► [LLM Parity]
   (Current)             Tier 1                Tier 2                  Tier 3                  Tier 4                 Tier 5               Tier 6

```

### Tier 1: Spatial Routing & Compositionality (From Isolated Gate to Circuit)

An XOR gate proves local non-linearity, but scalable computation requires signals to travel, branch, and interact without destructive interference.

#### Milestone 1.1: Signal Transport & Fan-Out

* **Capability:** Propagate a binary signal over distance $D$ with zero attenuation, and duplicate a single input into multiple independent downstream channels ($\text{Fan-Out} \ge 2$).
* **Evaluation Task:** 1-to-2 Buffer over a minimum spatial distance (e.g., $D \ge 30$ cells).
* **Success Metric:** $100\%$ transmission fidelity across both outputs under random background initialization or spatial jitter.

#### Milestone 1.2: Multi-Gate Composition & Wire Crossing

* **Capability:** Cascade multiple non-linear operations without intermediate signal degradation, including the resolution of non-planar signal crossings (two signal paths crossing without crosstalk).
* **Evaluation Task:** 1-bit Full Adder (Inputs: $A, B, C_{\text{in}}$; Outputs: $\text{Sum}, C_{\text{out}}$) or 2-bit Multiplier.
* **Success Metric:** Exact truth-table parity across all $2^n$ combinations, verified at steady-state or a fixed delay window.
* **Note (post-verification, see `docs/architecture/adr-0001-substrate-connectivity-topology.md`):** The planar wire-crossing sub-problem verified here was an artifact of choosing a strict 2D planar, nearest-neighbor-only embedding, not a requirement of the substrate itself. Future milestones are not required to reproduce this constraint or its shielding machinery.

### Tier 2: Temporal Dynamics & State Retention (From Static to Sequential Logic)

Combinational circuits are memoryless. Natural language and sequence modeling require temporal state retention that persists independently of immediate input stimulation.

#### Milestone 2.1: Bistable Latching (Dynamic Bit Storage)

* **Capability:** A localized spatial basin or dynamic cycle that retains state ($0$ or $1$) indefinitely after the input stimulus is removed, supporting controlled Write and Read triggers.
* **Evaluation Task:** Gated D-Latch or Set-Reset Flip-Flop.
* **Success Metric:** State retention over $\Delta t \ge 10^3$ substrate update steps post-write, with read accuracy $100\%$.

#### Milestone 2.2: Finite State Automata (Regular Languages)

* **Capability:** Sequential state tracking driven by a streaming input signal. The substrate transitions between internal states and classifies regular expressions.
* **Evaluation Task:** Deterministic Finite Automaton (DFA) recognition of a regular grammar (e.g., Even/Odd parity of an arbitrary-length bitstream, or recognizing the regex `(10)+1`).
* **Success Metric:** Zero classification error on streaming sequences of length $L \in [10, 100]$ tokens.

### Tier 3: Hierarchical Structure & Working Memory (Exceeding Finite State Limits)

Language is hierarchical, not merely Markovian. To reach LLM-grade capability, the system must navigate beyond Chomsky regular languages into context-free and associative dynamics.

#### Milestone 3.1: Pushdown Memory (Dyck Languages)

* **Capability:** Unbounded or dynamically expanding state memory that can match nested, long-range dependencies (stack/queue behavior).
* **Evaluation Task:** Dyck-1 (balanced parentheses `(()())`) and Dyck-2 (nested multi-bracket `([{}])`).
* **Success Metric:** Generalization to nesting depths $2\times$ deeper than the maximum depth observed during agent optimization.

#### Milestone 3.2: Associative Key-Value Retrieval ("Needle in a Haystack")

* **Capability:** Dynamic addressing. The substrate stores multiple variable bindings $(K_1 \to V_1, K_2 \to V_2)$ presented sequentially, ignores filler/distractor tokens, and extracts the target $V_i$ when probed with $K_i$.
* **Evaluation Task:** Stream $N$ key-value pairs separated by noise tokens, followed by a query token $K_j$. Output must emit $V_j$.
* **Success Metric:** Retrieval accuracy $\ge 99\%$ across variable context horizons ($N \ge 16$ pairs, sequence length $L \ge 256$).

### Tier 4: Statistical Sequence Modeling & Distributed Semantics

Real-world language modeling requires moving from brittle, discrete bit logic to distributed representations and probabilistic next-token prediction.

#### Milestone 4.1: Distributed Representations & Fault-Tolerant Basins

* **Capability:** Mapping discrete token spaces into distributed spatial configurations (hyperdimensional / semantic vector representations) where semantic proximity translates to attractor basin overlap.
* **Evaluation Task:** Nearest-neighbor classification under bit-flip noise (e.g., classifying corrupted 64-bit vector embeddings).
* **Success Metric:** Smooth degradation curve rather than catastrophic failure when noise increases from $5\%$ to $25\%$.

#### Milestone 4.2: Probabilistic Next-Token Prediction (Synthetic Grammars)

* **Capability:** Estimating the conditional probability distribution $P(x_t \mid x_{<t})$ over a finite vocabulary.
* **Evaluation Task:** Next-token prediction on a Probabilistic Context-Free Grammar (PCFG) or $k$-th order Hidden Markov model.
* **Success Metric:** Substrate prediction perplexity approaching the theoretical Shannon entropy limit $H(X)$ of the generating grammar.

### Tier 5: Attention-Like Dynamic Routing & In-Context Learning

The defining capability of modern LLMs is not just executing a fixed pre-compiled program, but acting as a programmable substrate whose runtime behavior is steered by context.

#### Milestone 5.1: The Induction Head Primitive

* **Capability:** Dynamic pattern completion on novel symbols. If the sequence has previously seen `[A] [B]` and later encounters `[A]`, the system routes internal information to predict `[B]`, even if `A` and `B` are random, unseen tokens.
* **Evaluation Task:** Two-pass random sequence completion: `... [X] [Y] ... [X] -> ?` (Target: `[Y]`).
* **Success Metric:** Dynamic copy accuracy $>95\%$ on completely novel, out-of-distribution token IDs.

#### Milestone 5.2: In-Context Task Switching (Few-Shot Prompting)

* **Capability:** The substrate dynamically reconfigures its effective input-output mapping based on a short sequence of demonstration examples without altering underlying cellular update rules.
* **Evaluation Task:** Present 3 to 5 input-output pairs of an arbitrary Boolean function or character substitution cipher, followed by a test query.
* **Success Metric:** Accuracy on the conditioned task significantly exceeding prior zero-shot probability ($p < 0.001$).

### Tier 6: Open-Domain Language Generation & Scaling Parity

The final horizon transitions the substrate from synthetic toy tasks to standardized natural language benchmarks.

#### Milestone 6.1: Micro-Language Modeling (BabyLM / TinyStories)

* **Capability:** Coherent autoregressive text generation across a fixed vocabulary (e.g., 1k–4k byte-pair encoded tokens).
* **Evaluation Task:** Autoregressive training on the TinyStories or BabyLM 10M dataset.
* **Success Metric:**

1. Grammar and syntactic coherence over a 128-token generation window.
2. Validation loss competitive with small baseline transformer architectures (e.g., 1M–5M parameter models).

#### Milestone 6.2: Substrate Scaling Laws

* **Capability:** Verifiable power-law scaling of compute, grid volume, and representation capacity against loss.
* **Evaluation Task:** Measuring cross-entropy loss against substrate scale ($S = \text{Grid Dimensions} \times \text{State Cardinality} \times \text{Temporal Relaxation Steps}$).
* **Success Metric:** Demonstrating a linear relationship on a log-log plot ($\mathcal{L} \propto S^{-\alpha}$), proving that performance scales smoothly with compute rather than plateauing.

### Benchmark & Capability Summary

| Tier | Milestone | Core Verification Target | Architectural Hurdle to Solve |
| --- | --- | --- | --- |
| **1. Combinational** | 1.1–1.2 | Full Adder / 2-bit Multiplier | Signal attenuation, non-planar wire crossing (retired as mandatory constraint, see ADR-0001) |
| **2. Sequential** | 2.1–2.2 | D-Latch & Regular Expression DFA | Sustained bistability, clocking/synchronization |
| **3. Memory** | 3.1–3.2 | Dyck Languages & Associative K-V | Stack allocation, long-range light-cone routing |
| **4. Statistical** | 4.1–4.2 | Noise-Tolerant Basins & PCFG Perplexity | Continuous/distributed states, probabilistic readout |
| **5. Cognitive** | 5.1–5.2 | Induction Heads & In-Context Few-Shot | Dynamic addressing, runtime meta-reconfiguration |
| **6. Parity** | 6.1–6.2 | TinyStories Perplexity & Scaling Laws | Parameter density, inference latency, compute scaling |

### Strategic Guidance for the Scientific Agent Team

To keep the agent team unconstrained yet focused, establish three operational ground rules:

1. **Black-Box Evaluator Interface:** All milestones must evaluate only the substrate's **Interface Contract** (`encode_input(tokens) -> run_dynamics(substrate) -> decode_output() -> Metric`). Never constrain whether the substrate uses continuous fields, discrete cell states, asynchronous updates, or reservoir readouts.
2. **The "Speed of Light" Bottleneck:** In standard local CAs, information travels at at most 1 cell per step. Reaching Tier 3 and beyond will require the team to discover mechanisms for rapid non-local communication (e.g., hierarchical grids, small-world connections, fast wave carriers, or multi-scale neighborhoods). Per `docs/architecture/adr-0001-substrate-connectivity-topology.md`, strict 2D planar embedding is not required: higher-dimensional lattices and bounded-degree long-range shortcut edges are permitted, provided locality and a fixed connection-degree budget are preserved (no unconstrained, zero-cost, all-to-all routing).
3. **Automated Promotion Gates:** The agent team should only be permitted to allocate compute to Tier $N+1$ once Tier $N$ benchmarks achieve their quantitative threshold across multiple randomized seeds.
