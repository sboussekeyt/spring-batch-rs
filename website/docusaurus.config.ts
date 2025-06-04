import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: "Spring Batch RS",
  tagline: "A toolkit for building enterprise-grade batch applications in Rust",
  favicon: "img/favicon.ico",

  // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
  },

  // Set the production url of your site here
  url: "https://sboussekeyt.github.io",
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: "/spring-batch-rs/",

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: "sboussekeyt", // Usually your GitHub org/user name.
  projectName: "spring-batch-rs", // Usually your repo name.

  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "warn",

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  // Enable Mermaid diagrams
  markdown: {
    mermaid: true,
  },
  themes: ["@docusaurus/theme-mermaid"],

  presets: [
    [
      "classic",
      {
        docs: {
          sidebarPath: "./sidebars.ts",
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            "https://github.com/sboussekeyt/spring-batch-rs/tree/main/website/",
        },
        blog: false, // Disable blog
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    // Replace with your project's social card
    image: "img/spring-batch-rs-social-card.jpg",
    // Mermaid theme configuration
    mermaid: {
      theme: { light: "neutral", dark: "dark" },
    },
    navbar: {
      title: "Spring Batch RS",
      logo: {
        alt: "Spring Batch RS Logo",
        src: "img/logo.svg",
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "tutorialSidebar",
          position: "left",
          label: "Documentation",
        },
        {
          href: "https://docs.rs/spring-batch-rs",
          label: "API Docs",
          position: "right",
        },
        {
          href: "https://crates.io/crates/spring-batch-rs",
          label: "Crates.io",
          position: "right",
        },
        {
          href: "https://github.com/sboussekeyt/spring-batch-rs",
          label: "GitHub",
          position: "right",
        },
      ],
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Docs",
          items: [
            {
              label: "Getting Started",
              to: "/docs/intro",
            },
            {
              label: "API Reference",
              href: "https://docs.rs/spring-batch-rs",
            },
          ],
        },
        {
          title: "Community",
          items: [
            {
              label: "Discord",
              href: "https://discord.gg/9FNhawNsG6",
            },
            {
              label: "GitHub Discussions",
              href: "https://github.com/sboussekeyt/spring-batch-rs/discussions",
            },
          ],
        },
        {
          title: "More",
          items: [
            {
              label: "GitHub",
              href: "https://github.com/sboussekeyt/spring-batch-rs",
            },
            {
              label: "Crates.io",
              href: "https://crates.io/crates/spring-batch-rs",
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Spring Batch RS. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ["rust", "toml"],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
