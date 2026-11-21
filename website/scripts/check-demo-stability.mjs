import assert from "node:assert/strict";
import { launch } from "./browser.mjs";
const browser = await launch();
try {
  for (const width of [390, 1440]) {
    const page = await browser.newPage({ viewport: { width, height: 900 } });
    await page.route("https://**/*", (route) => route.abort());
    await page.goto("http://127.0.0.1:4400/glance/");
    await page.waitForSelector("#demo");
    await page.evaluate(() => document.fonts.ready);
    await page.evaluate(() =>
      window.scrollTo(
        0,
        document.querySelector("#demo").getBoundingClientRect().top +
          scrollY -
          100,
      ),
    );
    await page.waitForTimeout(1000);
    const measure = () =>
      page.evaluate(() => ({
        scroll: scrollY,
        demo:
          document.querySelector("#demo").getBoundingClientRect().top + scrollY,
        facts:
          document.querySelector(".facts").getBoundingClientRect().top +
          scrollY,
        height: document.querySelector("#demo").getBoundingClientRect().height,
      }));
    const stable = async (label, action) => {
      const before = await measure();
      await action();
      await page.waitForTimeout(250);
      const after = await measure();
      console.log(JSON.stringify({ width, label, before, after }));
      for (const key of ["scroll", "demo", "facts", "height"])
        assert.ok(
          Math.abs(before[key] - after[key]) <= 1,
          `${width}px ${label}: ${key} jumped ${after[key] - before[key]}px`,
        );
    };
    const expand = page.locator("[data-demo-expand]").first();
    await stable("collapse", () => expand.click());
    await stable("expand", () => expand.click());
    const search = page.getByRole("textbox", { name: "Search apps or PID" });
    await stable("search", () => search.fill("19220"));
    await stable("empty search", () => search.fill("no-such-app"));
    await stable("clear search", () =>
      page.getByRole("button", { name: "Clear search" }).click(),
    );
    await stable("open sort", () =>
      page.getByRole("button", { name: "Sort processes" }).click(),
    );
    await stable("sort by memory", () =>
      page.getByRole("button", { name: "Sort by memory", exact: true }).click(),
    );
    const info = page
      .getByRole("button", { name: /Inspect details Firefox/ })
      .first();
    await stable("inspect", () => info.click());
    await stable("close inspector", () =>
      page.getByRole("button", { name: "Close", exact: true }).last().click(),
    );
    await stable("minimize", () =>
      page
        .getByRole("button", { name: "Minimize to tray", exact: true })
        .click(),
    );
    await stable("restore", () =>
      page.getByRole("button", { name: "Restore window", exact: true }).click(),
    );
    await stable("end demo app", () => page.getByRole("button",{name:"End Firefox in demo",exact:true}).click());
    await stable("reset demo", () => page.getByRole("button",{name:"Restore processes",exact:true}).first().click());
    const y = await page.evaluate(() => scrollY);
    await page.locator(".demo-scroll").hover();
    await page.mouse.wheel(0, 500);
    await page.waitForTimeout(250);
    assert.equal(
      await page.evaluate(() => scrollY),
      y,
      "Internal scrolling moved the page",
    );
    await page.close();
  }
} finally {
  await browser.close();
}
