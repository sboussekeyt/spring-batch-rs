# Website Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Simplify the homepage to 4 focused sections, surface the Java benchmark as the primary selling argument, fix all broken internal links, add sidebar navigation for missing pages, and add GitHub source links on every example page.

**Architecture:** Three independent files to modify: `index.mdx` (homepage rewrite), `sidebar.json` (navigation), and 7 example `.mdx` pages (GitHub source links). CSS for the new benchmark bars component goes into `landing.css`. No new components, no Astro config changes.

**Tech Stack:** Astro + Starlight MDX, Tailwind CSS, `@astrojs/starlight/components` (LinkCard, CardGrid, Aside)

---

## File Map

| File | Change |
|---|---|
| `website/src/styles/landing.css` | Add `.bench-*` CSS classes for benchmark bars |
| `website/src/content/docs/index.mdx` | Full rewrite: 4 sections, fix links, add benchmark |
| `website/src/config/sidebar.json` | Add Guide section + expand Reference section |
| `website/src/content/docs/examples/csv.mdx` | Add Aside import + GitHub source link |
| `website/src/content/docs/examples/json.mdx` | Add GitHub source link (Aside already imported) |
| `website/src/content/docs/examples/xml.mdx` | Add GitHub source link |
| `website/src/content/docs/examples/database.mdx` | Add GitHub source link |
| `website/src/content/docs/examples/mongodb.mdx` | Add GitHub source link |
| `website/src/content/docs/examples/tasklets.mdx` | Add GitHub source link (2 files: zip + ftp) |
| `website/src/content/docs/examples/advanced-patterns.mdx` | Add GitHub source link |

---

## Task 1: Add benchmark bar CSS to landing.css

**Files:**
- Modify: `website/src/styles/landing.css` (append at end)

- [ ] **Step 1: Append benchmark CSS classes**

Open `website/src/styles/landing.css` and append at the very end:

```css
/* =========================================================
   Benchmark comparison bars (homepage section ②)
   ========================================================= */

.bench-section {
  margin: 2.5rem 0;
  padding: 2rem;
  background: var(--sl-color-gray-6);
  border: 1px solid var(--sl-color-hairline);
  border-radius: 0.75rem;
}

.bench-section h2 {
  font-size: 1.5rem;
  font-weight: 800;
  margin: 0.25rem 0 0.375rem;
  line-height: 1.2;
}

.bench-tag {
  display: inline-block;
  font-size: 0.65rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--sl-color-gray-3);
}

.bench-subtitle {
  color: var(--sl-color-gray-3);
  font-size: 0.875rem;
  margin-bottom: 1.5rem;
  margin-top: 0;
}

.bench-grid {
  display: grid;
  gap: 0.875rem;
}

.bench-row {
  display: grid;
  grid-template-columns: 8rem 1fr 13rem;
  gap: 0.75rem;
  align-items: center;
}

@media (max-width: 640px) {
  .bench-row {
    grid-template-columns: 1fr;
    gap: 0.2rem;
    margin-bottom: 0.5rem;
  }
}

.bench-label {
  font-size: 0.8rem;
  color: var(--sl-color-gray-2);
}

.bench-track {
  height: 1.5rem;
  background: var(--sl-color-gray-5);
  border-radius: 0.25rem;
  display: flex;
  overflow: hidden;
}

.bench-bar-rust {
  background: #238636;
  flex-shrink: 0;
  border-radius: 0.25rem 0 0 0.25rem;
  display: flex;
  align-items: center;
  padding: 0 0.4rem;
  font-size: 0.7rem;
  font-weight: 700;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  min-width: 0;
}

.bench-bar-java {
  background: #da3633;
  opacity: 0.45;
  flex: 1;
  border-radius: 0 0.25rem 0.25rem 0;
}

.bench-delta {
  font-size: 0.8rem;
  color: var(--sl-color-gray-2);
  text-align: right;
}

.bench-delta strong {
  color: #3fb950;
}

.bench-legend {
  display: flex;
  gap: 1.25rem;
  margin-top: 1.25rem;
  font-size: 0.75rem;
  color: var(--sl-color-gray-3);
}

.bench-legend-item {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.bench-dot {
  width: 0.625rem;
  height: 0.625rem;
  border-radius: 0.125rem;
  display: inline-block;
  flex-shrink: 0;
}

.bench-dot-rust { background: #238636; }
.bench-dot-java { background: #da3633; opacity: 0.6; }

.bench-link {
  display: inline-block;
  margin-top: 1rem;
  font-size: 0.8rem;
  color: var(--sl-color-accent);
  text-decoration: underline;
}

.showcase-source-link {
  font-size: 0.8rem;
  color: var(--sl-color-accent);
  text-decoration: underline;
}
```

- [ ] **Step 2: Verify CSS parses (quick build check)**

```bash
cd website && npm run build 2>&1 | tail -5
```

Expected: no CSS errors. If you see a parse error, check for unclosed braces in the block you added.

- [ ] **Step 3: Commit**

```bash
git add website/src/styles/landing.css
git commit -m "style: add benchmark bar CSS classes to landing.css"
```

---

## Task 2: Rewrite index.mdx

**Files:**
- Modify: `website/src/content/docs/index.mdx`

The current file has 7 sections with broken `/spring-batch-rs/` link prefixes. This task applies
targeted edits: fix hero links, delete 4 sections, add the benchmark section, update the nav grid.

- [ ] **Step 1: Fix hero action links in the frontmatter**

In `website/src/content/docs/index.mdx`, find:

```yaml
  actions:
    - text: Get Started →
      link: /spring-batch-rs/getting-started/
      icon: right-arrow
      variant: primary
    - text: View Examples
      link: /spring-batch-rs/quick-examples/
      icon: document
      variant: minimal
```

Replace with:

```yaml
  actions:
    - text: Get Started →
      link: /getting-started/
      icon: right-arrow
      variant: primary
    - text: View Examples
      link: /quick-examples/
      icon: document
      variant: minimal
```

- [ ] **Step 2: Update the tagline to be more direct**

Find:

```yaml
  tagline: Spring Batch patterns you know. Rust performance you need.
```

Replace with:

```yaml
  tagline: Spring Batch patterns you know. Rust performance you need. No GC. No surprises.
```

- [ ] **Step 3: Update the import line**

Find:

```
import { Card, CardGrid, LinkCard } from "@astrojs/starlight/components";
```

Replace with:

```
import { LinkCard, CardGrid } from "@astrojs/starlight/components";
```

- [ ] **Step 4: Delete the problem-solution block**

Delete the entire block from `<div class="problem-solution">` through its closing `</div>` (inclusive). It ends just before `<div class="tech-specs">`. The block spans lines 23–32 in the current file.

The block to delete looks like:

```html
<div class="problem-solution">
  <div class="problem-block">
    ...
  </div>
  <div class="solution-block">
    ...
  </div>
</div>
```

- [ ] **Step 5: Delete the tech-specs block**

Delete the entire `<div class="tech-specs">...</div>` block (4 spec-card divs). It starts with `<div class="tech-specs">` and ends with its closing `</div>` (line ~98 in current file).

- [ ] **Step 6: Delete the integration-matrix block**

Delete the entire `<div class="integration-matrix">...</div>` block. It starts with `<div class="integration-matrix">` and ends with its closing `</div>` (around line 152).

- [ ] **Step 7: Add the benchmark section before the code-showcase block**

Find the line:

```html
<div class="code-showcase">
```

Insert the following block immediately before it (leave a blank line between the new block and the code-showcase div):

```html
<div class="bench-section">
  <span class="bench-tag">Performance · benchmark reproductible</span>
  <h2>4.5× faster than Spring Batch Java</h2>
  <p class="bench-subtitle">10 million financial transactions · CSV → PostgreSQL → XML · same chunk size, same connection pool</p>
  <div class="bench-grid">
    <div class="bench-row">
      <span class="bench-label">Total pipeline</span>
      <div class="bench-track">
        <div class="bench-bar-rust" style="width: 22.5%">42s</div>
        <div class="bench-bar-java"></div>
      </div>
      <span class="bench-delta"><strong>4.5×</strong> faster · 187s</span>
    </div>
    <div class="bench-row">
      <span class="bench-label">Peak memory</span>
      <div class="bench-track">
        <div class="bench-bar-rust" style="width: 3.4%"></div>
        <div class="bench-bar-java"></div>
      </div>
      <span class="bench-delta"><strong>30×</strong> less · 62 MB vs 1 840 MB</span>
    </div>
    <div class="bench-row">
      <span class="bench-label">Cold start</span>
      <div class="bench-track">
        <div class="bench-bar-rust" style="width: 0.5%; min-width: 8px"></div>
        <div class="bench-bar-java"></div>
      </div>
      <span class="bench-delta"><strong>320×</strong> faster · &lt;10ms vs 3.2s</span>
    </div>
  </div>
  <div class="bench-legend">
    <span class="bench-legend-item"><span class="bench-dot bench-dot-rust"></span>Spring Batch RS (Rust)</span>
    <span class="bench-legend-item"><span class="bench-dot bench-dot-java"></span>Spring Batch (Java)</span>
  </div>
  <a href="/reference/java-vs-rust-benchmark/" class="bench-link">→ Methodology, full numbers &amp; how to reproduce</a>
</div>
```

- [ ] **Step 8: Update the code-showcase header and footer**

Find in the showcase-header:

```html
    <h2>CSV → JSON Pipeline</h2>
    <p>Transform data in just a few lines of elegant, type-safe code</p>
```

Replace with:

```html
    <h2>CSV → JSON Pipeline</h2>
    <p>Type-safe, fault-tolerant, zero boilerplate</p>
```

Find the showcase-footer (contains 3 footer-metric divs):

```html
  <div class="showcase-footer">
    <div class="footer-metric">
      <span class="metric-icon">⚙️</span>
      <span>Type-Safe</span>
    </div>
    <div class="footer-metric">
      <span class="metric-icon">⚡</span>
      <span>Zero-Cost</span>
    </div>
    <div class="footer-metric">
      <span class="metric-icon">🛡️</span>
      <span>Fault Tolerant</span>
    </div>
  </div>
```

Replace with:

```html
  <div class="showcase-footer">
    <a href="https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/csv_processing.rs" class="showcase-source-link">→ View full source on GitHub</a>
  </div>
```

- [ ] **Step 9: Delete the terminal-cta block**

Delete the entire `<div class="terminal-cta">...</div>` block. It starts with `<div class="terminal-cta">` and ends with its closing `</div>` (around lines 220–268).

- [ ] **Step 10: Replace the resource-grid LinkCards**

Find the entire `<div class="resource-grid">...</div>` block:

```html
<div class="resource-grid">
  <LinkCard
    title="API Reference"
    description="Complete API documentation with examples"
    href="/spring-batch-rs/api/"
  />
  <LinkCard
    title="Architecture Guide"
    description="Core concepts & design patterns"
    href="/spring-batch-rs/architecture/"
  />
  <LinkCard
    title="Quick Start"
    description="Get up and running in 5 minutes"
    href="/spring-batch-rs/getting-started/"
  />
  <LinkCard
    title="Examples Gallery"
    description="24+ real-world code examples"
    href="/spring-batch-rs/quick-examples/"
  />
</div>
```

Replace with:

```html
<CardGrid>
  <LinkCard
    title="Getting Started"
    description="First batch job in 5 minutes. Installation, setup, first example."
    href="/getting-started/"
  />
  <LinkCard
    title="Java vs Rust Benchmark"
    description="Methodology, full numbers & how to reproduce on your own infrastructure."
    href="/reference/java-vs-rust-benchmark/"
  />
  <LinkCard
    title="Examples Gallery"
    description="24+ pipelines with links to source code on GitHub."
    href="/quick-examples/"
  />
  <LinkCard
    title="Architecture"
    description="Job → Step → Reader/Processor/Writer. Chunk-oriented processing."
    href="/architecture/"
  />
</CardGrid>
```

- [ ] **Step 11: Build and verify no errors**

```bash
cd website && npm run build 2>&1 | grep -E "error|Error" | head -20
```

Expected: zero errors. If you see an MDX parse error, the most likely cause is an unclosed HTML tag — verify all deleted blocks were fully removed (opening `<div>` and closing `</div>` both gone).

- [ ] **Step 12: Smoke-test locally**

```bash
make website-serve
```

Open http://localhost:4321 and verify:
- The hero has 2 buttons: "Get Started" and "View Examples"
- The benchmark bars section appears immediately below the hero
- The code-showcase block renders with the rust code example
- The footer shows "→ View full source on GitHub" link
- 4 LinkCards appear at the bottom (Getting Started, Benchmark, Examples, Architecture)
- Inspect the page source — confirm zero occurrences of `/spring-batch-rs/` in href attributes

- [ ] **Step 13: Commit**

```bash
git add website/src/content/docs/index.mdx
git commit -m "feat(website): rewrite homepage — 4 sections, benchmark bars, fix broken links"
```

---

## Task 3: Update sidebar navigation

**Files:**
- Modify: `website/src/config/sidebar.json` (full replacement)

The current sidebar has no Guide section. Pages like Getting Started, Architecture, and the Benchmark are unreachable from the sidebar.

- [ ] **Step 1: Replace sidebar.json**

```json
{
  "main": [
    {
      "label": "Guide",
      "items": [
        {
          "label": "Getting Started",
          "link": "/getting-started/"
        },
        {
          "label": "Architecture",
          "link": "/architecture/"
        },
        {
          "label": "Processing Models",
          "link": "/processing-models/"
        },
        {
          "label": "Error Handling",
          "link": "/error-handling/"
        }
      ]
    },
    {
      "label": "Examples",
      "items": [
        {
          "label": "[book-open] Overview",
          "link": "/examples/"
        },
        {
          "label": "[table] CSV",
          "link": "/examples/csv/"
        },
        {
          "label": "[code] JSON",
          "link": "/examples/json/"
        },
        {
          "label": "[file-code] XML",
          "link": "/examples/xml/"
        },
        {
          "label": "[database] Database",
          "link": "/examples/database/"
        },
        {
          "label": "[globe] MongoDB",
          "link": "/examples/mongodb/"
        },
        {
          "label": "[layers] ORM",
          "link": "/examples/orm/"
        },
        {
          "label": "[terminal] Tasklets",
          "link": "/examples/tasklets/"
        },
        {
          "label": "[zap] Advanced Patterns",
          "link": "/examples/advanced-patterns/"
        }
      ]
    },
    {
      "label": "Tasklets",
      "autogenerate": { "directory": "tasklets" }
    },
    {
      "label": "Tutorials",
      "autogenerate": { "directory": "tutorials" }
    },
    {
      "label": "Reference",
      "items": [
        {
          "label": "[zap] Java vs Rust Benchmark",
          "link": "/reference/java-vs-rust-benchmark/"
        },
        {
          "label": "[list] Features",
          "link": "/reference/features/"
        },
        {
          "label": "[alert-circle] Error Types",
          "link": "/reference/error-types/"
        },
        {
          "label": "[external-link] API Documentation",
          "link": "https://docs.rs/spring-batch-rs",
          "badge": "external"
        }
      ]
    }
  ]
}
```

- [ ] **Step 2: Build and verify**

```bash
cd website && npm run build 2>&1 | grep -E "error|Error" | head -10
```

Expected: no errors. If Starlight warns about unknown icon names (e.g. `[alert-circle]`), replace with a known icon like `[information]`.

- [ ] **Step 3: Verify sidebar renders**

```bash
make website-serve
```

Open http://localhost:4321 and check the left sidebar contains:
- **Guide** section with Getting Started, Architecture, Processing Models, Error Handling
- **Examples** section (unchanged items)
- **Reference** section with Java vs Rust Benchmark at the top

- [ ] **Step 4: Commit**

```bash
git add website/src/config/sidebar.json
git commit -m "feat(website): add Guide section and Benchmark to sidebar navigation"
```

---

## Task 4: Add GitHub source links to example pages

**Files:**
- Modify: `website/src/content/docs/examples/csv.mdx`
- Modify: `website/src/content/docs/examples/json.mdx`
- Modify: `website/src/content/docs/examples/xml.mdx`
- Modify: `website/src/content/docs/examples/database.mdx`
- Modify: `website/src/content/docs/examples/mongodb.mdx`
- Modify: `website/src/content/docs/examples/tasklets.mdx`
- Modify: `website/src/content/docs/examples/advanced-patterns.mdx`

Each file gets an `<Aside type="tip">` block with the GitHub link, inserted after the import line(s) and before the first paragraph. The base URL for all links is: `https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/`

**csv.mdx** — special case: `Aside` is not yet imported.

- [ ] **Step 1: Edit csv.mdx — add Aside to import and insert link**

Find this line at the top of `website/src/content/docs/examples/csv.mdx`:
```
import { Tabs, TabItem, Card, CardGrid } from '@astrojs/starlight/components';
```
Replace with:
```
import { Tabs, TabItem, Card, CardGrid, Aside } from '@astrojs/starlight/components';
```

Then find:
```
This page provides comprehensive examples for working with CSV files using Spring Batch RS.
```
Insert before it:
```mdx
<Aside type="tip">
  View the complete source: [examples/csv_processing.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/csv_processing.rs)
</Aside>
```

- [ ] **Step 2: Edit json.mdx**

Find:
```
This page provides comprehensive examples for working with JSON files using Spring Batch RS.
```
Insert before it:
```mdx
<Aside type="tip">
  View the complete source: [examples/json_processing.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/json_processing.rs)
</Aside>
```

- [ ] **Step 3: Edit xml.mdx**

Find:
```
This page provides comprehensive examples for working with XML files using Spring Batch RS.
```
Insert before it:
```mdx
<Aside type="tip">
  View the complete source: [examples/xml_processing.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/xml_processing.rs)
</Aside>
```

- [ ] **Step 4: Edit database.mdx**

Find:
```
This page provides comprehensive examples for working with relational databases (PostgreSQL, MySQL, SQLite) using Spring Batch RS.
```
Insert before it:
```mdx
<Aside type="tip">
  View the complete source: [examples/database_processing.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/database_processing.rs)
</Aside>
```

- [ ] **Step 5: Edit mongodb.mdx**

Find:
```
This page provides comprehensive examples for working with MongoDB using Spring Batch RS.
```
Insert before it:
```mdx
<Aside type="tip">
  View the complete source: [examples/mongodb_processing.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/mongodb_processing.rs)
</Aside>
```

- [ ] **Step 6: Edit tasklets.mdx**

Find:
```
This page provides comprehensive examples for using tasklets in Spring Batch RS for file operations and single-task processing.
```
Insert before it:
```mdx
<Aside type="tip">
  View the complete sources: [examples/tasklet_zip.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/tasklet_zip.rs) · [examples/tasklet_ftp.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/tasklet_ftp.rs)
</Aside>
```

- [ ] **Step 7: Edit advanced-patterns.mdx**

Find:
```
# Advanced Patterns
```
Insert before it:
```mdx
<Aside type="tip">
  View the complete source: [examples/advanced_patterns.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/advanced_patterns.rs)
</Aside>
```

- [ ] **Step 8: Build and verify all 7 pages**

```bash
cd website && npm run build 2>&1 | grep -E "error|Error" | head -10
```

Expected: no errors.

- [ ] **Step 9: Spot-check in browser**

```bash
make website-serve
```

Open http://localhost:4321/examples/csv/ — verify the green tip aside with the GitHub link appears at the top of the page content. Check one other page (e.g. `/examples/tasklets/`) to confirm the two-link aside renders correctly.

- [ ] **Step 10: Commit**

```bash
git add website/src/content/docs/examples/csv.mdx \
        website/src/content/docs/examples/json.mdx \
        website/src/content/docs/examples/xml.mdx \
        website/src/content/docs/examples/database.mdx \
        website/src/content/docs/examples/mongodb.mdx \
        website/src/content/docs/examples/tasklets.mdx \
        website/src/content/docs/examples/advanced-patterns.mdx
git commit -m "feat(website): add GitHub source links to all example pages"
```

---

## Task 5: Final verification

- [ ] **Step 1: Full production build**

```bash
make website-build 2>&1 | tail -15
```

Expected: build succeeds with no errors. Note the final "Built in X.Xs" line.

- [ ] **Step 2: Check no broken /spring-batch-rs/ links remain**

```bash
grep -r "spring-batch-rs/" website/src/content/docs/index.mdx
```

Expected: no output (all internal links have been corrected).

- [ ] **Step 3: Verify success criteria from spec**

- [ ] No broken links from the homepage (all hrefs start with `/` not `/spring-batch-rs/`)
- [ ] Benchmark bars visible in section ② without scrolling on 1080p
- [ ] Every example page has a GitHub source link
- [ ] Architecture, Getting Started, Benchmark appear in sidebar navigation
- [ ] `make website-build` succeeds with no errors
