# Website Redesign — Design Spec

**Date:** 2026-04-10
**Status:** Approved

---

## Context

The spring-batch-rs documentation site (`https://spring-batch-rs.boussekeyt.dev`) has four known problems:

1. **Broken links** — `index.mdx` uses a `/spring-batch-rs/` URL prefix (leftover from a GitHub Pages base path) but `astro.config.mjs` has no `base` option. Every internal link in the homepage is broken.
2. **Benchmark buried** — A real, reproducible Java vs Rust benchmark exists (`examples/benchmark_csv_postgres_xml.rs` + `benchmark/java/`) but is not surfaced from the homepage.
3. **No source links on examples pages** — Example pages don't link to their corresponding `.rs` files on GitHub.
4. **Homepage too dense** — 7 sections of equal visual weight make it hard to scan.

**Target audience:** Both Java developers (familiar with Spring Batch, evaluating migration) and Rust developers (looking for a batch framework). Equal weight.

**Homepage goal:** Quick "wow" section at the top + arguments below.

---

## Design Decisions

### Homepage structure (chosen: Option A — Hero + Immediate impact)

4 sections, in order:

```
① HERO
② BENCHMARK BARS (Java vs Rust)
③ CODE EXAMPLE
④ NAVIGATION GRID
```

**Sections removed from current homepage:**
- Problem/Solution block (redundant with hero tagline)
- Tech specs × 4 cards (No GC, Memory Safety, Zero-cost, Type-safe) — the benchmark section communicates this more concretely
- Integration matrix (moved to Getting Started page)
- Terminal CTA (merged into the nav grid card)

### Benchmark section design (chosen: Option C — Bar chart)

Three horizontal bar charts, Rust (green) vs Java (red), for:
- Total pipeline time: 42s vs 187s → **4.5×**
- Peak memory (RSS): 62 MB vs 1 840 MB → **30×**
- Cold start: <10ms vs 3 200ms → **320×**

Link to the full benchmark page below the chart.

Benchmark factuality: The code is real (`examples/benchmark_csv_postgres_xml.rs` and `benchmark/java/`). The numbers are presented as reference measurements with a disclaimer to reproduce on your own infrastructure. This is honest — do not remove the disclaimer.

### Navigation grid

4 cards at the bottom of the homepage:
- Getting Started
- **Java vs Rust Benchmark** (highlighted in green — the selling argument)
- Examples Gallery
- Architecture

---

## Files to Change

### 1. `website/src/content/docs/index.mdx` — Full rewrite

**Structure:**
```
frontmatter (keep hero section, fix action links)
② benchmark bars section (new HTML block)
③ code example (simplified from current, add GitHub link)
④ nav grid (4 LinkCards, fix hrefs)
```

**Link fixes required (all occurrences):**

| Current (broken) | Fixed |
|---|---|
| `/spring-batch-rs/getting-started/` | `/getting-started/` |
| `/spring-batch-rs/quick-examples/` | `/quick-examples/` |
| `/spring-batch-rs/api/` | `/api/` |
| `/spring-batch-rs/architecture/` | `/architecture/` |

### 2. `website/src/config/sidebar.json` — Add missing pages

Pages currently missing from navigation:
- Getting Started (`/getting-started/`)
- Architecture (`/architecture/`)
- Java vs Rust Benchmark (`/reference/java-vs-rust-benchmark/`)
- Error Handling (`/error-handling/`)
- Processing Models (`/processing-models/`)

Proposed sidebar structure:

```json
[
  { "label": "Guide", "items": [
    { "label": "Getting Started", "link": "/getting-started/" },
    { "label": "Architecture", "link": "/architecture/" },
    { "label": "Processing Models", "link": "/processing-models/" },
    { "label": "Error Handling", "link": "/error-handling/" }
  ]},
  { "label": "Examples", "items": [ ...existing... ] },
  { "label": "Reference", "items": [
    { "label": "Java vs Rust Benchmark", "link": "/reference/java-vs-rust-benchmark/" },
    { "label": "Features", "link": "/reference/features/" },
    { "label": "Error Types", "link": "/reference/error-types/" },
    { "label": "API Documentation (docs.rs)", "link": "https://docs.rs/spring-batch-rs", "badge": "external" }
  ]}
]
```

### 3. Example pages — Add GitHub source links

Each example `.mdx` file should include a GitHub link to its corresponding `.rs` file. The GitHub repo is `https://github.com/sboussekeyt/spring-batch-rs`.

Mapping:
| Page | Source file |
|---|---|
| `examples/csv.mdx` | `examples/csv_processing.rs` |
| `examples/json.mdx` | `examples/json_processing.rs` |
| `examples/xml.mdx` | `examples/xml_processing.rs` |
| `examples/database.mdx` | `examples/database_processing.rs` |
| `examples/mongodb.mdx` | `examples/mongodb_processing.rs` |
| `examples/tasklets.mdx` | `examples/tasklet_zip.rs`, `examples/tasklet_ftp.rs` |
| `examples/advanced-patterns.mdx` | `examples/advanced_patterns.rs` |

Each link should appear as a badge/aside near the top of the page, e.g.:
```mdx
import { Aside } from '@astrojs/starlight/components';
<Aside type="tip">
  View the full source: [examples/csv_processing.rs](https://github.com/sboussekeyt/spring-batch-rs/blob/main/examples/csv_processing.rs)
</Aside>
```

---

## Out of Scope

- Redesigning individual doc pages (architecture, getting-started, etc.) — content is kept as-is
- Adding new benchmark data or re-running the benchmark
- i18n / French translations
- Any Astro component changes beyond `index.mdx` and `sidebar.json`

---

## Success Criteria

- [ ] No broken links from the homepage
- [ ] Benchmark bars visible in section ② without scrolling on a 1080p screen
- [ ] Every example page has a GitHub source link
- [ ] Architecture, Getting Started, Benchmark appear in sidebar navigation
- [ ] `make website-build` succeeds with no errors
