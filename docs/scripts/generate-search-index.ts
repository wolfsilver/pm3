import {
  mkdirSync,
  readdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative } from "node:path";
import { createSearchAPI } from "fumadocs-core/search/server";

const CONTENT_DIR = join(import.meta.dirname, "../content/docs");
const OUTPUT_FILE = join(import.meta.dirname, "../public/search-index.json");

interface Frontmatter {
  title?: string;
  description?: string;
  keywords?: string;
}

function parseFrontmatter(fileContent: string): {
  frontmatter: Frontmatter;
  body: string;
} {
  const match = fileContent.match(/^---\n([\s\S]*?)\n---\n?/);
  if (!match) {
    return {
      frontmatter: {},
      body: fileContent,
    };
  }

  const frontmatterBlock = match[1];
  if (frontmatterBlock === undefined) {
    return {
      frontmatter: {},
      body: fileContent,
    };
  }

  const frontmatter: Frontmatter = {};
  for (const line of frontmatterBlock.split("\n")) {
    const separatorIndex = line.indexOf(":");
    if (separatorIndex === -1) continue;

    const key = line.slice(0, separatorIndex).trim();
    const value = line
      .slice(separatorIndex + 1)
      .trim()
      .replace(/^["']|["']$/g, "");

    if (key === "title" || key === "description" || key === "keywords") {
      frontmatter[key] = value;
    }
  }

  return {
    frontmatter,
    body: fileContent.slice(match[0].length),
  };
}

function humanizeSlugPart(value: string): string {
  return value
    .split(/[-_]/g)
    .filter(Boolean)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

function deriveTitle(slug: string): string {
  const part = slug.split("/").filter(Boolean).at(-1) ?? "docs";
  return humanizeSlugPart(part);
}

function toDocsUrl(filePath: string): string {
  const relativePath = relative(CONTENT_DIR, filePath).replace(/\\/g, "/");
  const withoutExtension = relativePath.replace(/\.mdx?$/, "");
  const withoutIndex = withoutExtension.replace(/\/index$/, "");

  if (withoutIndex === "" || withoutIndex === "index") {
    return "/docs";
  }

  return `/docs/${withoutIndex}`;
}

function toBreadcrumbs(url: string): string[] {
  const parts = url
    .replace(/^\/docs\/?/, "")
    .split("/")
    .filter(Boolean)
    .map(humanizeSlugPart);

  return parts;
}

function sanitizeContent(markdown: string): string {
  return markdown
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/~~~[\s\S]*?~~~/g, " ")
    .replace(/!\[([^\]]*)\]\([^)]+\)/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/<[^>]+>/g, " ")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/^#{1,6}\s+/gm, "")
    .replace(/^\s*[-*+]\s+/gm, "")
    .replace(/^\s*\d+\.\s+/gm, "")
    .replace(/\s+/g, " ")
    .trim();
}

function collectDocFiles(dir: string): string[] {
  const files: string[] = [];

  for (const entry of readdirSync(dir)) {
    const fullPath = join(dir, entry);
    const stat = statSync(fullPath);

    if (stat.isDirectory()) {
      files.push(...collectDocFiles(fullPath));
    } else if (entry.endsWith(".mdx") || entry.endsWith(".md")) {
      files.push(fullPath);
    }
  }

  return files;
}

async function main() {
  const files = collectDocFiles(CONTENT_DIR);
  const indexes = files.map((file) => {
    const raw = readFileSync(file, "utf-8");
    const { frontmatter, body } = parseFrontmatter(raw);

    const url = toDocsUrl(file);
    const slug = url.replace(/^\/docs\/?/, "");
    const title = frontmatter.title ?? deriveTitle(slug);
    const description = frontmatter.description;
    const keywords = frontmatter.keywords;
    const content = sanitizeContent(body);
    const breadcrumbs = toBreadcrumbs(url);

    return {
      title,
      description,
      content,
      url,
      breadcrumbs,
      keywords,
    };
  });

  const searchAPI = createSearchAPI("simple", { indexes });
  const data = await searchAPI.export();

  mkdirSync(dirname(OUTPUT_FILE), { recursive: true });
  writeFileSync(OUTPUT_FILE, JSON.stringify(data));

  console.log(`Generated search index for ${indexes.length} docs.`);
  console.log(`Output: ${OUTPUT_FILE}`);
}

main().catch((error: unknown) => {
  console.error("Failed to generate static search index:", error);
  process.exit(1);
});
