# Homepage Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rewrite the documentation homepage to clearly answer "why Spring Batch in Rust?" with a structured, sober layout that speaks to both Rust developers and Java/Spring developers.

**Architecture:** The homepage is `website/src/content/docs/index.mdx` (Astro + Starlight, `template: splash`). Styles live in `website/src/styles/landing.css`, which is imported via `global.css`. No Rust code is touched — this is a pure documentation/website change.

**Tech Stack:** Astro 5, Starlight, Tailwind CSS v4, MDX

---

## Pre-flight

Before starting, start the dev server to preview changes live:

```bash
cd website && npm run dev
# Visit http://localhost:4321/spring-batch-rs/
```

To verify the build is clean at any point:

```bash
cd website && npm run build
```

---

## Task 1: Update the Hero tagline

**Files:**
- Modify: `website/src/content/docs/index.mdx` (lines 1–19)

**Context:** The current tagline is generic. Replace it with a one-liner that speaks to both audiences.

**Step 1: Edit the hero frontmatter**

In `website/src/content/docs/index.mdx`, replace lines 1–19 with:

```mdx
---
title: Spring Batch RS
description: Spring Batch patterns you know. Rust performance you need.
template: splash
hero:
  title: |
    Enterprise Batch Processing
    <span class="hero-gradient">Powered by Rust</span>
  tagline: Spring Batch patterns you know. Rust performance you need.
  actions:
    - text: Get Started →
      link: /spring-batch-rs/getting-started/
      icon: right-arrow
      variant: primary
    - text: View Examples
      link: /spring-batch-rs/quick-examples/
      icon: document
      variant: minimal
---
```

**Step 2: Visual check**

Open http://localhost:4321/spring-batch-rs/ and verify:
- The tagline reads "Spring Batch patterns you know. Rust performance you need."
- The gradient title and buttons are unchanged

**Step 3: Commit**

```bash
git add website/src/content/docs/index.mdx
git commit -m "docs(website): update homepage hero tagline"
```

---

## Task 2: Remove the pipeline-flow stats bar

**Files:**
- Modify: `website/src/content/docs/index.mdx`

**Context:** The three-node bar (PERFORMANCE / RELIABILITY / EXTENSIBLE) provides no explanation — it's pure decoration. Remove it.

**Step 1: Delete the pipeline-flow block**

In `index.mdx`, delete this entire block (currently starts after the frontmatter imports):

```mdx
<div class="pipeline-flow">
  ...
</div>
```

(The block spans from `<div class="pipeline-flow">` to its closing `</div>`, roughly lines 23–54.)

**Step 2: Visual check**

The decorative stat bar should be gone. The page should jump directly to the first content section.

**Step 3: Commit**

```bash
git add website/src/content/docs/index.mdx
git commit -m "docs(website): remove decorative pipeline-flow bar"
```

---

## Task 3: Add the "Problem → Solution" section

**Files:**
- Modify: `website/src/content/docs/index.mdx`
- Modify: `website/src/styles/landing.css`

**Context:** This is the new section that explains *why* the framework exists. It replaces nothing — it's inserted before the tech specs.

**Step 1: Add CSS for the problem section**

Append the following to the end of `website/src/styles/landing.css`:

```css
/* Problem → Solution Section */
.problem-solution {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 3rem;
  margin: 4rem auto;
  padding: 3rem;
  background: rgba(26, 31, 46, 0.4);
  border: 1px solid rgba(0, 217, 255, 0.15);
  border-radius: 1.5rem;
  animation: fadeInUp 0.8s ease-out 0.3s both;
}

.problem-solution .problem-block {
  padding-right: 2rem;
  border-right: 1px solid rgba(0, 217, 255, 0.15);
}

.problem-solution .solution-block {
  padding-left: 1rem;
}

.problem-solution .block-label {
  display: inline-block;
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  padding: 0.3rem 0.75rem;
  border-radius: 2rem;
  margin-bottom: 1.25rem;
}

.problem-block .block-label {
  background: rgba(255, 80, 80, 0.1);
  border: 1px solid rgba(255, 80, 80, 0.3);
  color: #ff6b6b;
}

.solution-block .block-label {
  background: rgba(0, 217, 255, 0.1);
  border: 1px solid rgba(0, 217, 255, 0.3);
  color: #00d9ff;
}

.problem-solution p {
  color: var(--text-color-text);
  line-height: 1.75;
  font-size: 1.05rem;
  margin: 0;
}

@media (max-width: 768px) {
  .problem-solution {
    grid-template-columns: 1fr;
    gap: 2rem;
    padding: 2rem;
  }

  .problem-block {
    border-right: none !important;
    padding-right: 0 !important;
    border-bottom: 1px solid rgba(0, 217, 255, 0.15);
    padding-bottom: 2rem;
  }

  .solution-block {
    padding-left: 0 !important;
  }
}
```

**Step 2: Add the problem section to index.mdx**

After the opening `import` line and before `<div class="tech-specs">`, insert:

```mdx
<div class="problem-solution">
  <div class="problem-block">
    <span class="block-label">The Problem</span>
    <p>Batch jobs fail silently. Memory grows until the process crashes. Errors get swallowed. Retries are manual. Monitoring is an afterthought. Rolling your own framework means reinventing the same hard parts every time.</p>
  </div>
  <div class="solution-block">
    <span class="block-label">The Solution</span>
    <p>Spring Batch RS brings proven patterns from the Java ecosystem — chunk-oriented processing, skip &amp; retry policies, job lifecycle tracking — rewritten in Rust for memory safety and predictable performance. No GC. No surprises.</p>
  </div>
</div>
```

**Step 3: Visual check**

The two-column block should appear below the hero, with a red "THE PROBLEM" label on the left and a cyan "THE SOLUTION" label on the right.

**Step 4: Commit**

```bash
git add website/src/content/docs/index.mdx website/src/styles/landing.css
git commit -m "docs(website): add problem/solution section to homepage"
```

---

## Task 4: Replace Tech Specs with "Why Rust for batch processing?"

**Files:**
- Modify: `website/src/content/docs/index.mdx`

**Context:** The current `.tech-specs` block has three cards. The primary card displays fake metrics ("10M+ Records/sec", "<50MB Memory"). Replace the entire block with four honest, concept-focused cards.

**Step 1: Replace the tech-specs block**

Find and replace the entire `<div class="tech-specs">...</div>` block with:

```mdx
<div class="tech-specs">
  <div class="spec-card spec-primary">
    <div class="spec-header">
      <div class="spec-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 6v6l4 2"/>
        </svg>
      </div>
      <div class="spec-meta">
        <span class="spec-tag">RUNTIME</span>
        <h3>No GC Pauses</h3>
      </div>
    </div>
    <p>Rust's ownership model means no garbage collector to interrupt your batch jobs at runtime. Your throughput is consistent — no pause-the-world surprises at 3 AM.</p>
  </div>

  <div class="spec-card">
    <div class="spec-header">
      <div class="spec-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
        </svg>
      </div>
      <div class="spec-meta">
        <span class="spec-tag">SAFETY</span>
        <h3>Memory Safety</h3>
      </div>
    </div>
    <p>Buffer overflows and null pointer errors are compile-time failures, not production incidents. The compiler catches entire classes of bugs before they reach your data.</p>
  </div>

  <div class="spec-card">
    <div class="spec-header">
      <div class="spec-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="12 2 2 7 12 12 22 7 12 2"/>
          <polyline points="2 17 12 22 22 17"/>
          <polyline points="2 12 12 17 22 12"/>
        </svg>
      </div>
      <div class="spec-meta">
        <span class="spec-tag">ABSTRACTIONS</span>
        <h3>Zero-Cost Abstractions</h3>
      </div>
    </div>
    <p>The chunk-oriented pipeline, builder patterns, and trait-based extensibility add no overhead beyond what you explicitly write. High-level code, systems-level performance.</p>
  </div>

  <div class="spec-card">
    <div class="spec-header">
      <div class="spec-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2"/>
          <path d="M9 9h6v6H9z"/>
        </svg>
      </div>
      <div class="spec-meta">
        <span class="spec-tag">TYPE SAFETY</span>
        <h3>Type-Safe Pipelines</h3>
      </div>
    </div>
    <p>Your reader, processor, and writer types must match at compile time. Connect a <code>CsvReader&lt;Product&gt;</code> to a <code>JsonWriter&lt;Order&gt;</code> and it simply won't compile.</p>
  </div>
</div>
```

**Step 2: Visual check**

Four cards should appear, with titles: No GC Pauses, Memory Safety, Zero-Cost Abstractions, Type-Safe Pipelines. No fake numbers anywhere.

**Step 3: Commit**

```bash
git add website/src/content/docs/index.mdx
git commit -m "docs(website): replace tech specs with honest 'why rust' cards"
```

---

## Task 5: Update the code showcase comments

**Files:**
- Modify: `website/src/content/docs/index.mdx`

**Context:** The existing code example is correct but the comments don't explain *concepts* — they just state what the code does. Improve comments so a reader unfamiliar with chunk-oriented processing understands what `chunk(100)` and `skip_limit(10)` mean.

**Step 1: Replace the code block inside `.code-showcase`**

Find the existing ` ```rust ` block and replace it with:

````mdx
```rust
use spring_batch_rs::prelude::*;

// 1. Define your data shape — Rust enforces type safety at compile time
#[derive(Deserialize, Serialize)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

// 2. Configure source and destination using builder patterns
let reader = CsvItemReaderBuilder::<Product>::new()
    .from_path("products.csv")
    .has_headers(true)
    .build();

let writer = JsonItemWriterBuilder::<Product>::new()
    .from_path("products.json");

// 3. Build the pipeline
//    chunk(100)      → accumulate 100 items, then write once (balances I/O vs memory)
//    skip_limit(10)  → tolerate up to 10 bad records before the job fails
let step = StepBuilder::new("convert-products")
    .reader(&reader)
    .writer(&writer)
    .chunk(100)
    .skip_limit(10)
    .build();

// 4. Run — the job tracks status, counts, and errors automatically
JobBuilder::new().start(&step).build().run();
```
````

**Step 2: Visual check**

The code block should show the improved comments inline with the code.

**Step 3: Commit**

```bash
git add website/src/content/docs/index.mdx
git commit -m "docs(website): improve code example comments for clarity"
```

---

## Task 6: Remove the "Enterprise Capabilities" grid

**Files:**
- Modify: `website/src/content/docs/index.mdx`

**Context:** The `.capabilities-grid` section (6 cards: Retry & Skip, Async-First, Extensible Traits, Logging, Battle-Tested, Developer Friendly) completely overlaps with the new "Why Rust" cards (Task 4) and the integration matrix. Delete the entire block.

**Step 1: Delete the capabilities-grid block**

Remove the entire `<div class="capabilities-grid">...</div>` block from `index.mdx`.

**Step 2: Visual check**

The section titled "Enterprise Capabilities" should no longer appear on the page.

**Step 3: Commit**

```bash
git add website/src/content/docs/index.mdx
git commit -m "docs(website): remove redundant enterprise capabilities section"
```

---

## Task 7: Remove the "Production Use Cases" section

**Files:**
- Modify: `website/src/content/docs/index.mdx`

**Context:** The `.use-case-section` (Data Migration, ETL Pipelines, Report Generation, Data Import/Export) is generic and redundant with the problem section added in Task 3. Delete the entire block.

**Step 1: Delete the use-case-section block**

Remove the entire `<div class="use-case-section">...</div>` block from `index.mdx`.

**Step 2: Visual check**

The section titled "Production Use Cases" should no longer appear.

**Step 3: Commit**

```bash
git add website/src/content/docs/index.mdx
git commit -m "docs(website): remove redundant use cases section"
```

---

## Task 8: Simplify the ecosystem section header

**Files:**
- Modify: `website/src/content/docs/index.mdx`

**Context:** The integration matrix title is "Integration Ecosystem" — make it more direct. Also add "Utilities" to the matrix to surface zip/ftp/fake features.

**Step 1: Update the matrix title**

Find `<h2>Integration Ecosystem</h2>` and replace with:

```html
<h2>Works With Your Stack</h2>
```

**Step 2: Add a Utilities column**

Inside `.matrix-grid`, after the Runtime section, add:

```mdx
    <div class="matrix-section">
      <div class="matrix-header">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
        </svg>
        <span>Utilities</span>
      </div>
      <div class="matrix-items">
        <span class="matrix-tag">Tokio Async</span>
        <span class="matrix-tag">Fault Tolerance</span>
        <span class="matrix-tag">ZIP</span>
        <span class="matrix-tag">FTP</span>
        <span class="matrix-tag">Fake Data</span>
      </div>
    </div>
```

**Step 3: Visual check**

The matrix should show four columns: Databases / Formats / Runtime / Utilities.

**Step 4: Commit**

```bash
git add website/src/content/docs/index.mdx
git commit -m "docs(website): update ecosystem section title and add utilities"
```

---

## Task 9: Final build verification

**Step 1: Run the production build**

```bash
cd website && npm run build
```

Expected: build completes with no errors. Warnings about unused CSS classes are acceptable.

**Step 2: Preview the production build**

```bash
cd website && npm run preview
# Visit http://localhost:4321/spring-batch-rs/
```

Scroll through the full page and verify this reading order:
1. Hero: "Spring Batch patterns you know. Rust performance you need."
2. Problem/Solution two-column block
3. "Why Rust for batch processing?" — 4 cards, no fake numbers
4. Code example with explanatory comments
5. "Works With Your Stack" — integration matrix with 4 columns
6. Terminal CTA (`cargo add spring-batch-rs`)
7. LinkCards (API Reference, Architecture, Quick Start, Examples)

**Step 3: Commit if any last-minute fixes were made**

```bash
git add -p
git commit -m "docs(website): homepage redesign final tweaks"
```

---

## Summary of Changes

| File | Type | What changed |
|---|---|---|
| `website/src/content/docs/index.mdx` | Modified | New tagline, problem section, 4 honest why-rust cards, improved code comments, removed 2 redundant sections, updated matrix |
| `website/src/styles/landing.css` | Modified | Added `.problem-solution` styles |

## Sections Before → After

| Before | After |
|---|---|
| Decorative pipeline-flow bar | Removed |
| Generic tagline | "Spring Batch patterns you know. Rust performance you need." |
| *(nothing)* | Problem → Solution two-column block |
| Tech specs with fake metrics | 4 honest "Why Rust" cards |
| Code example (sparse comments) | Code example (concept-explaining comments) |
| "Enterprise Capabilities" (6 cards) | Removed (redundant) |
| "Production Use Cases" (4 cards) | Removed (redundant) |
| "Integration Ecosystem" (3 cols) | "Works With Your Stack" (4 cols) |
