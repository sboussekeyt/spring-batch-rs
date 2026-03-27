// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import { viewTransitions } from "astro-vtbot/starlight-view-transitions";
import astroMermaid from "astro-mermaid";

import tailwindcss from "@tailwindcss/vite";
import config from "./src/config/config.json" assert { type: "json" };
import social from "./src/config/social.json";
import locals from "./src/config/locals.json";
import sidebar from "./src/config/sidebar.json";

import { fileURLToPath } from "url";

const { site } = config;
const { title } = site;

export const locales = locals;

// https://astro.build/config
export default defineConfig({
  site: "https://spring-batch-rs.boussekeyt.dev",
  image: {
    service: { entrypoint: "astro/assets/services/noop" },
  },
  integrations: [
    astroMermaid({
      theme: "base",
      darkModeTheme: "dark",
    }),
    starlight({
      title,
      description:
        "A comprehensive toolkit for building enterprise-grade batch applications in Rust",
      // @ts-ignore
      social: social.main || [],
      locales,
      sidebar: sidebar.main || [],
      customCss: ["./src/styles/global.css"],
      components: {
        Head: "./src/components/override-components/Head.astro",
        Header: "./src/components/override-components/Header.astro",
        Hero: "./src/components/override-components/Hero.astro",
        PageFrame: "./src/components/override-components/PageFrame.astro",
        PageSidebar: "./src/components/override-components/PageSidebar.astro",
        TwoColumnContent:
          "./src/components/override-components/TwoColumnContent.astro",
        ContentPanel: "./src/components/override-components/ContentPanel.astro",
        Pagination: "./src/components/override-components/Pagination.astro",
        Sidebar: "./src/components/override-components/Sidebar.astro",
      },
    }),
  ],
  vite: {
    plugins: [tailwindcss(), viewTransitions()],
    resolve: {
      alias: {
        "@": fileURLToPath(new URL("./src", import.meta.url)),
        "~": fileURLToPath(new URL("./src", import.meta.url)),
      },
    },
  },
});
