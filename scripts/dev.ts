#!/usr/bin/env bun
import { spawn } from "bun";

const CYAN = "\x1b[36m";
const MAGENTA = "\x1b[35m";
const RESET = "\x1b[0m";

function streamPrefixed(stream: ReadableStream<Uint8Array>, prefix: string, color: string) {
  const reader = stream.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  (async () => {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split("\n");
      buffer = lines.pop() ?? "";
      for (const line of lines) {
        process.stdout.write(`${color}[${prefix}]${RESET} ${line}\n`);
      }
    }
    if (buffer) {
      process.stdout.write(`${color}[${prefix}]${RESET} ${buffer}\n`);
    }
  })().catch(() => {});
}

// 1. Rust API (Axum on port 8080)
const api = spawn(["sh", "-c", "set -a; [ -f .env ] && . ./.env; set +a; cargo run -p arrhabon-api"], {
  cwd: process.cwd(),
  stdout: "pipe",
  stderr: "pipe",
  env: {
    ...process.env,
    PATH: `/Users/prabalpatra/Library/pnpm:/opt/homebrew/bin:${process.env.PATH}`,
  },
});

streamPrefixed(api.stdout, "api", CYAN);
streamPrefixed(api.stderr, "api", CYAN);

// 2. Next.js Web (Port 3000 via Bun)
const web = spawn(["bun", "run", "dev", "--", "-p", "3000"], {
  cwd: "web",
  stdout: "pipe",
  stderr: "pipe",
  env: {
    ...process.env,
    PORT: "3000",
  },
});

streamPrefixed(web.stdout, "web", MAGENTA);
streamPrefixed(web.stderr, "web", MAGENTA);

let exiting = false;
function cleanup() {
  if (exiting) return;
  exiting = true;
  try {
    api.kill();
  } catch {}
  try {
    web.kill();
  } catch {}
  process.exit(0);
}

process.on("SIGINT", cleanup);
process.on("SIGTERM", cleanup);

await Promise.race([api.exited, web.exited]);
cleanup();
