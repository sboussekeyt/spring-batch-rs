---
title: Examples Gallery
description: Complete examples for all Spring Batch RS features — CSV, JSON, XML, databases, tasklets, and advanced patterns.
sidebar:
  order: 0
---

<style>
.eg-header { margin-bottom: 1.75rem; }
.eg-description { color: var(--sl-color-gray-2); font-size: 1rem; line-height: 1.6; margin: 0 0 1.25rem; }
.eg-stats { display: flex; align-items: center; gap: 1rem; flex-wrap: wrap; margin-bottom: 1.75rem; }
.eg-stat { display: flex; flex-direction: column; gap: 0.1rem; }
.eg-stat-num { font-size: 1.4rem; font-weight: 700; color: var(--sl-color-white); line-height: 1; }
.eg-stat-label { font-size: 0.68rem; font-weight: 600; letter-spacing: 0.08em; text-transform: uppercase; color: var(--sl-color-gray-3); }
.eg-stat-sep { width: 1px; height: 2rem; background: var(--sl-color-gray-6); }

.eg-grid { display: grid; gap: 1rem; grid-template-columns: 1fr; margin-top: 0 !important; }
@media (min-width: 600px) { .eg-grid { grid-template-columns: repeat(2, 1fr); } }
@media (min-width: 1024px) { .eg-grid { grid-template-columns: repeat(4, 1fr); gap: 1.1rem; } }

.eg-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
  padding: 1.1rem;
  border-radius: 0.875rem;
  border: 1px solid var(--sl-color-gray-6);
  background: var(--sl-color-bg);
  text-decoration: none !important;
  overflow: hidden;
  transition: border-color 0.25s ease, transform 0.25s ease, box-shadow 0.25s ease;
}
.eg-card::before {
  content: "";
  position: absolute;
  top: 0; left: 0; right: 0;
  height: 2px;
  background: var(--cat);
  opacity: 0.55;
  transition: opacity 0.25s ease;
}
.eg-card::after {
  content: "";
  position: absolute;
  top: -50px; right: -50px;
  width: 130px; height: 130px;
  border-radius: 50%;
  background: var(--cat);
  opacity: 0;
  filter: blur(45px);
  transition: opacity 0.35s ease;
  pointer-events: none;
}
.eg-card:hover { border-color: color-mix(in srgb, var(--cat) 45%, transparent); transform: translateY(-2px); box-shadow: 0 8px 32px color-mix(in srgb, var(--cat) 8%, transparent); }
.eg-card:hover::before { opacity: 1; }
.eg-card:hover::after { opacity: 0.14; }

.eg-card-top { display: flex; align-items: center; justify-content: space-between; }
.eg-icon { display: flex; align-items: center; justify-content: center; width: 2.4rem; height: 2.4rem; border-radius: 0.5rem; background: color-mix(in srgb, var(--cat) 14%, transparent); color: var(--cat); flex-shrink: 0; transition: background 0.25s; }
.eg-card:hover .eg-icon { background: color-mix(in srgb, var(--cat) 22%, transparent); }
.eg-badge { font-size: 0.58rem; font-weight: 700; letter-spacing: 0.12em; text-transform: uppercase; color: var(--cat); background: color-mix(in srgb, var(--cat) 11%, transparent); border: 1px solid color-mix(in srgb, var(--cat) 28%, transparent); padding: 0.18rem 0.42rem; border-radius: 0.25rem; white-space: nowrap; }

.eg-title { font-size: 0.95rem; font-weight: 700; color: var(--sl-color-white); margin: 0; line-height: 1.3; border: none !important; padding: 0 !important; }
.eg-desc { font-size: 0.8rem; color: var(--sl-color-gray-3); line-height: 1.5; margin: 0; flex: 1; }

.eg-features { display: flex; flex-wrap: wrap; gap: 0.3rem; }
.eg-tag { font-size: 0.62rem; font-family: var(--sl-font-mono, monospace); color: var(--sl-color-gray-3); background: var(--sl-color-gray-6); padding: 0.13rem 0.38rem; border-radius: 0.22rem; letter-spacing: 0.02em; }

.eg-cta { display: flex; align-items: center; gap: 0.4rem; font-size: 0.78rem; font-weight: 600; color: var(--cat); margin-top: auto; }
.eg-cta-arrow { transition: transform 0.2s ease; display: inline-block; }
.eg-card:hover .eg-cta-arrow { transform: translateX(4px); }

.eg-commands { margin-top: 2rem; }
</style>

<div class="eg-header">
  <p class="eg-description">Browse examples organized by feature. Each page includes documentation and links to runnable source code on GitHub.</p>
  <div class="eg-stats">
    <div class="eg-stat"><span class="eg-stat-num">8</span><span class="eg-stat-label">Categories</span></div>
    <div class="eg-stat-sep"></div>
    <div class="eg-stat"><span class="eg-stat-num">24+</span><span class="eg-stat-label">Examples</span></div>
    <div class="eg-stat-sep"></div>
    <div class="eg-stat"><span class="eg-stat-num">100%</span><span class="eg-stat-label">Source on GitHub</span></div>
  </div>
</div>

<div class="eg-grid">

  <a href="/examples/csv/" class="eg-card" style="--cat: #10b981">
    <div class="eg-card-top">
      <div class="eg-icon">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M3 15h18M9 3v18"/></svg>
      </div>
      <span class="eg-badge">FORMAT</span>
    </div>
    <p class="eg-title">CSV Processing</p>
    <p class="eg-desc">Read and write CSV files with full header support, custom delimiters, and fault-tolerant parsing.</p>
    <div class="eg-features"><span class="eg-tag">csv</span><span class="eg-tag">json</span><span class="eg-tag">serde</span></div>
    <div class="eg-cta">View examples <span class="eg-cta-arrow">→</span></div>
  </a>

  <a href="/examples/json/" class="eg-card" style="--cat: #3b82f6">
    <div class="eg-card-top">
      <div class="eg-icon">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H7a2 2 0 0 0-2 2v5a2 2 0 0 1-2 2 2 2 0 0 1 2 2v5a2 2 0 0 0 2 2h1"/><path d="M16 3h1a2 2 0 0 1 2 2v5a2 2 0 0 0 2 2 2 2 0 0 0-2 2v5a2 2 0 0 1-2 2h-1"/></svg>
      </div>
      <span class="eg-badge">FORMAT</span>
    </div>
    <p class="eg-title">JSON Processing</p>
    <p class="eg-desc">Stream JSON arrays, write with pretty-printing, and transform between formats seamlessly.</p>
    <div class="eg-features"><span class="eg-tag">json</span><span class="eg-tag">csv</span><span class="eg-tag">serde</span></div>
    <div class="eg-cta">View examples <span class="eg-cta-arrow">→</span></div>
  </a>

  <a href="/examples/xml/" class="eg-card" style="--cat: #f59e0b">
    <div class="eg-card-top">
      <div class="eg-icon">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
      </div>
      <span class="eg-badge">FORMAT</span>
    </div>
    <p class="eg-title">XML Processing</p>
    <p class="eg-desc">Parse XML documents with namespace support, handle nested elements and attributes.</p>
    <div class="eg-features"><span class="eg-tag">xml</span><span class="eg-tag">json</span><span class="eg-tag">csv</span></div>
    <div class="eg-cta">View examples <span class="eg-cta-arrow">→</span></div>
  </a>

  <a href="/examples/database/" class="eg-card" style="--cat: #8b5cf6">
    <div class="eg-card-top">
      <div class="eg-icon">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14a9 3 0 0 0 18 0V5"/><path d="M3 12a9 3 0 0 0 18 0"/></svg>
      </div>
      <span class="eg-badge">DATABASE</span>
    </div>
    <p class="eg-title">Database (RDBC)</p>
    <p class="eg-desc">PostgreSQL, MySQL, and SQLite operations with paginated reads and efficient batch inserts.</p>
    <div class="eg-features"><span class="eg-tag">rdbc-postgres</span><span class="eg-tag">rdbc-sqlite</span></div>
    <div class="eg-cta">View examples <span class="eg-cta-arrow">→</span></div>
  </a>

  <a href="/examples/mongodb/" class="eg-card" style="--cat: #14b8a6">
    <div class="eg-card-top">
      <div class="eg-icon">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2C8 2 6 6 6 9c0 4.5 4 7 6 13 2-6 6-8.5 6-13 0-3-2-7-6-7z"/></svg>
      </div>
      <span class="eg-badge">DATABASE</span>
    </div>
    <p class="eg-title">MongoDB</p>
    <p class="eg-desc">Query collections, batch inserts, and cursor-based pagination for document databases.</p>
    <div class="eg-features"><span class="eg-tag">mongodb</span><span class="eg-tag">csv</span><span class="eg-tag">json</span></div>
    <div class="eg-cta">View examples <span class="eg-cta-arrow">→</span></div>
  </a>

  <a href="/examples/orm/" class="eg-card" style="--cat: #ec4899">
    <div class="eg-card-top">
      <div class="eg-icon">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="8" height="8" rx="1"/><rect x="14" y="3" width="8" height="8" rx="1"/><rect x="2" y="13" width="8" height="8" rx="1"/><rect x="14" y="13" width="8" height="8" rx="1"/></svg>
      </div>
      <span class="eg-badge">DATABASE</span>
    </div>
    <p class="eg-title">ORM (SeaORM)</p>
    <p class="eg-desc">Type-safe entity mapping, async queries, and seamless pagination with SeaORM integration.</p>
    <div class="eg-features"><span class="eg-tag">orm</span><span class="eg-tag">csv</span><span class="eg-tag">json</span></div>
    <div class="eg-cta">View examples <span class="eg-cta-arrow">→</span></div>
  </a>

  <a href="/examples/tasklets/" class="eg-card" style="--cat: #ef4444">
    <div class="eg-card-top">
      <div class="eg-icon">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20a8 8 0 1 0 0-16 8 8 0 0 0 0 16z"/><path d="M12 14a2 2 0 1 0 0-4 2 2 0 0 0 0 4z"/><path d="M12 2v2M12 20v2M2 12h2M20 12h2"/></svg>
      </div>
      <span class="eg-badge">OPERATIONS</span>
    </div>
    <p class="eg-title">Tasklets</p>
    <p class="eg-desc">ZIP compression, FTP file transfers, and other single-task operations outside chunk processing.</p>
    <div class="eg-features"><span class="eg-tag">zip</span><span class="eg-tag">ftp</span></div>
    <div class="eg-cta">View examples <span class="eg-cta-arrow">→</span></div>
  </a>

  <a href="/examples/advanced-patterns/" class="eg-card" style="--cat: #00d9ff">
    <div class="eg-card-top">
      <div class="eg-icon">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M6 9v6M9 6h9a3 3 0 0 1 3 3v6"/></svg>
      </div>
      <span class="eg-badge">PATTERNS</span>
    </div>
    <p class="eg-title">Advanced Patterns</p>
    <p class="eg-desc">Multi-step ETL pipelines, error recovery, processor chains, and complex production workflows.</p>
    <div class="eg-features"><span class="eg-tag">csv</span><span class="eg-tag">json</span><span class="eg-tag">logger</span></div>
    <div class="eg-cta">View examples <span class="eg-cta-arrow">→</span></div>
  </a>

</div>

## Quick Commands

Run any example directly with `cargo`:

```bash
cargo run --example csv_processing --features csv,json
cargo run --example json_processing --features json,csv,logger
cargo run --example xml_processing --features xml,json,csv
cargo run --example database_processing --features rdbc-sqlite,csv,json,logger
cargo run --example orm_processing --features orm,csv,json
cargo run --example tasklet_zip --features zip
cargo run --example advanced_patterns --features csv,json,logger
```

See [`Cargo.toml`](https://github.com/sboussekeyt/spring-batch-rs/blob/main/Cargo.toml) for the full list of examples and their required feature flags.
