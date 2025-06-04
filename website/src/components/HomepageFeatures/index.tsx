import type { ReactNode } from "react";
import clsx from "clsx";
import Heading from "@theme/Heading";
import styles from "./styles.module.css";
import React from "react";

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<"svg">>;
  description: React.JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: "Chunk-Oriented Processing",
    Svg: require("@site/static/img/chunk-processing.svg").default,
    description: (
      <>
        Process large datasets efficiently with the read-process-write pattern.
        Built-in support for pagination, error handling, and transaction
        boundaries.
      </>
    ),
  },
  {
    title: "Tasklet Processing",
    Svg: require("@site/static/img/tasklet-processing.svg").default,
    description: (
      <>
        Execute single tasks like file operations, database maintenance, or
        custom business logic that don't fit the chunk-oriented model.
      </>
    ),
  },
  {
    title: "Rich Ecosystem",
    Svg: require("@site/static/img/ecosystem.svg").default,
    description: (
      <>
        Support for CSV, JSON, XML, MongoDB, PostgreSQL, MySQL, SQLite, and
        more. Modular design lets you enable only what you need.
      </>
    ),
  },
];

function Feature({ title, Svg, description }: FeatureItem) {
  return (
    <div className={clsx("col col--4")}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): React.JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
