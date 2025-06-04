import type { ReactNode } from "react";
import clsx from "clsx";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import HomepageFeatures from "@site/src/components/HomepageFeatures";
import Heading from "@theme/Heading";
import React from "react";

import styles from "./index.module.css";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx("hero hero--primary", styles.heroBanner)}>
      <div className="container">
        <div className={styles.heroContent}>
          <div className={styles.heroText}>
            <Heading as="h1" className={clsx("hero__title", styles.heroTitle)}>
              {siteConfig.title}
            </Heading>
            <p className={clsx("hero__subtitle", styles.heroSubtitle)}>
              {siteConfig.tagline}
            </p>
            <div className={styles.heroDescription}>
              <p>
                Build robust, scalable batch processing applications with Rust's
                performance and safety. Inspired by Spring Batch, designed for
                modern enterprise workloads.
              </p>
            </div>
            <div className={styles.buttons}>
              <Link
                className={clsx(
                  "button button--primary button--lg",
                  styles.getStartedButton
                )}
                to="/docs/intro"
              >
                Get Started
              </Link>
              <Link
                className={clsx(
                  "button button--outline button--lg",
                  styles.githubButton
                )}
                to="https://github.com/sboussekeyt/spring-batch-rs"
              >
                View on GitHub
              </Link>
            </div>
          </div>
          <div className={styles.heroCode}>
            <div className={styles.codeBlock}>
              <div className={styles.codeHeader}>
                <div className={styles.codeTitle}>Quick Example</div>
                <div className={styles.codeDots}>
                  <span></span>
                  <span></span>
                  <span></span>
                </div>
              </div>
              <pre className={styles.codeContent}>
                <code>{`use spring_batch_rs::prelude::*;

#[derive(Deserialize, Serialize)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

fn main() -> Result<(), BatchError> {
    let reader = CsvItemReaderBuilder::<Product>::new()
        .has_headers(true)
        .from_path("products.csv");

    let writer = JsonItemWriterBuilder::new()
        .pretty(true)
        .from_path("output.json");

    let step = StepBuilder::new("csv_to_json")
        .chunk(100)
        .reader(&reader)
        .writer(&writer)
        .build();

    let job = JobBuilder::new()
        .start(&step)
        .build();
        
    job.run()
}`}</code>
              </pre>
            </div>
          </div>
        </div>
      </div>
      <div className={styles.heroWave}>
        <svg viewBox="0 0 1200 120" preserveAspectRatio="none">
          <path
            d="M0,0V46.29c47.79,22.2,103.59,32.17,158,28,70.36-5.37,136.33-33.31,206.8-37.5C438.64,32.43,512.34,53.67,583,72.05c69.27,18,138.3,24.88,209.4,13.08,36.15-6,69.85-17.84,104.45-29.34C989.49,25,1113-14.29,1200,52.47V0Z"
            opacity=".25"
          ></path>
          <path
            d="M0,0V15.81C13,36.92,27.64,56.86,47.69,72.05,99.41,111.27,165,111,224.58,91.58c31.15-10.15,60.09-26.07,89.67-39.8,40.92-19,84.73-46,130.83-49.67,36.26-2.85,70.9,9.42,98.6,31.56,31.77,25.39,62.32,62,103.63,73,40.44,10.79,81.35-6.69,119.13-24.28s75.16-39,116.92-43.05c59.73-5.85,113.28,22.88,168.9,38.84,30.2,8.66,59,6.17,87.09-7.5,22.43-10.89,48-26.93,60.65-49.24V0Z"
            opacity=".5"
          ></path>
          <path d="M0,0V5.63C149.93,59,314.09,71.32,475.83,42.57c43-7.64,84.23-20.12,127.61-26.46,59-8.63,112.48,12.24,165.56,35.4C827.93,77.22,886,95.24,951.2,90c86.53-7,172.46-45.71,248.8-84.81V0Z"></path>
        </svg>
      </div>
    </header>
  );
}

function StatsSection() {
  return (
    <section className={styles.statsSection}>
      <div className="container">
        <div className={styles.statsGrid}>
          <div className={styles.statItem}>
            <div className={styles.statNumber}>🚀</div>
            <div className={styles.statLabel}>High Performance</div>
            <div className={styles.statDescription}>
              Built with Rust for maximum performance and memory safety
            </div>
          </div>
          <div className={styles.statItem}>
            <div className={styles.statNumber}>📊</div>
            <div className={styles.statLabel}>Enterprise Ready</div>
            <div className={styles.statDescription}>
              Production-tested patterns for large-scale batch processing
            </div>
          </div>
          <div className={styles.statItem}>
            <div className={styles.statNumber}>🔧</div>
            <div className={styles.statLabel}>Extensible</div>
            <div className={styles.statDescription}>
              Modular design with custom readers, writers, and processors
            </div>
          </div>
          <div className={styles.statItem}>
            <div className={styles.statNumber}>🛡️</div>
            <div className={styles.statLabel}>Fault Tolerant</div>
            <div className={styles.statDescription}>
              Built-in error handling with configurable skip limits
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function QuickLinksSection() {
  return (
    <section className={styles.quickLinksSection}>
      <div className="container">
        <div className={styles.quickLinksHeader}>
          <Heading as="h2">Quick Links</Heading>
          <p>Jump right into what you need</p>
        </div>
        <div className={styles.quickLinksGrid}>
          <Link to="/docs/intro" className={styles.quickLinkCard}>
            <div className={styles.quickLinkIcon}>📚</div>
            <div className={styles.quickLinkContent}>
              <h3>Documentation</h3>
              <p>Complete guides and API reference</p>
            </div>
          </Link>
          <Link to="/docs/examples" className={styles.quickLinkCard}>
            <div className={styles.quickLinkIcon}>💡</div>
            <div className={styles.quickLinkContent}>
              <h3>Examples</h3>
              <p>Real-world usage patterns and code samples</p>
            </div>
          </Link>
          <Link
            to="https://crates.io/crates/spring-batch-rs"
            className={styles.quickLinkCard}
          >
            <div className={styles.quickLinkIcon}>📦</div>
            <div className={styles.quickLinkContent}>
              <h3>Crates.io</h3>
              <p>Download and installation instructions</p>
            </div>
          </Link>
          <Link
            to="https://docs.rs/spring-batch-rs"
            className={styles.quickLinkCard}
          >
            <div className={styles.quickLinkIcon}>🔍</div>
            <div className={styles.quickLinkContent}>
              <h3>API Docs</h3>
              <p>Detailed API documentation and references</p>
            </div>
          </Link>
        </div>
      </div>
    </section>
  );
}

export default function Home(): React.JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={`${siteConfig.title} - Enterprise Batch Processing for Rust`}
      description="A comprehensive toolkit for building enterprise-grade batch applications in Rust. Inspired by Spring Batch with modern Rust performance and safety."
    >
      <HomepageHeader />
      <main>
        <StatsSection />
        <HomepageFeatures />
        <QuickLinksSection />
      </main>
    </Layout>
  );
}
