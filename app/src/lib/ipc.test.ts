import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

/**
 * The command names are written twice — once in Rust, once in the calls
 * below — and nothing at compile time connects the two. A name misspelled
 * here fails at runtime, in the one place a user notices: a button that
 * does nothing.
 *
 * So the two lists are compared here instead. This is the TypeScript half
 * of the check `commands.rs` makes on the Rust side.
 */

function read(relative: string): string {
  return readFileSync(fileURLToPath(new URL(relative, import.meta.url)), "utf8");
}

/** The names in `booker_core::ipc::COMMANDS`, read from the contract. */
function contractCommands(): string[] {
  const source = read("../../../crates/booker-core/src/ipc.rs");
  const list = /pub const COMMANDS: &\[&str\] = &\[([^\]]*)\]/.exec(source);
  expect(list, "COMMANDS is no longer where this test looks for it").not.toBeNull();
  return [...list![1].matchAll(/"([a-z_]+)"/g)].map((match) => match[1]);
}

/** The names `ipc.ts` actually invokes. */
function invokedCommands(): string[] {
  const source = read("./ipc.ts");
  return [...source.matchAll(/invoke<[^>]*>\(\s*"([a-z_]+)"/g)].map((match) => match[1]);
}

describe("the commands this window calls", () => {
  it("are all in the Rust contract", () => {
    const contract = contractCommands();
    const invoked = invokedCommands();

    expect(invoked.length, "no invoke() calls found — has ipc.ts changed shape?").toBeGreaterThan(0);
    for (const name of invoked) {
      expect(contract, `"${name}" is not in booker_core::ipc::COMMANDS`).toContain(name);
    }
  });

  it("read a contract that is sorted, so a merge of two tracks stays textual", () => {
    const contract = contractCommands();
    expect(contract).toStrictEqual([...contract].sort());
  });
});

describe("the page image URL", () => {
  /**
   * The URL shape is written twice too — `page_image_url` in Rust builds
   * it, `protocol.rs` reads it, and `ipc.ts` builds it again for the
   * preview. A drift here shows up as a preview of blank pages, with no
   * error anywhere, so it is worth a test of its own.
   */
  it("is spelled the same in TypeScript as in the Rust contract", async () => {
    const rust = read("../../../crates/booker-core/src/ipc.rs");
    const template = /format!\("(booker:\/\/[^"]+)"\)/.exec(rust);
    expect(template, "page_image_url no longer builds its URL with format!").not.toBeNull();

    // Turn the Rust format string into what it produces for known values.
    const expected = template![1]
      .replace("{revision}", "7")
      .replace("{page}", "3")
      .replace("{scale}", "2")
      .replace("{format}", "png");

    const { pageImageUrl } = await import("./ipc");
    expect(pageImageUrl(7n, 3, 2, "png")).toBe(expected);
  });
});
