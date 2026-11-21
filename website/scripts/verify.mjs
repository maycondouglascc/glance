import fs from "node:fs/promises";
import { launch } from "./browser.mjs";
const browser = await launch();
const results = [];
await fs.mkdir(".qa", { recursive: true });
try {
  for (const width of [390, 768, 1440, 1920]) {
    for (const route of ["", "?page=guide", "?page=404"]) {
      const page = await browser.newPage({
        viewport: { width, height: 900 },
        reducedMotion: "reduce",
        locale: "en-US",
      });
      await page.route("https://**/*", (r) => r.abort());
      const errors = [];
      page.on("pageerror", (e) => errors.push(e.message));
      await page.goto("http://127.0.0.1:4400/glance/" + route);
      await page.waitForSelector("h1");
      await page.evaluate(() => document.fonts.ready);
      const name = (route.replace("?page=", "") || "home") + "-" + width;
      await page.screenshot({ path: ".qa/" + name + ".png", fullPage: true });
      await page.addScriptTag({ path: "node_modules/axe-core/axe.min.js" });
      const violations = await page.evaluate(async () =>
        (await axe.run()).violations.map((v) => ({
          id: v.id,
          impact: v.impact,
          nodes: v.nodes.map((n) => n.target),
        })),
      );
      const overflow = await page.evaluate(
        () => document.documentElement.scrollWidth > innerWidth,
      );
      results.push({ name, overflow, errors, violations });
      if (!route) {
        const search = page.getByRole("textbox", {
          name: "Search apps or PID",
        });
        await search.fill("19220");
        if (!(await page.locator(".glance-demo").innerText()).includes("19220"))
          throw Error("Process search failed");
        await search.fill("");
        await page.locator("[data-demo-expand]").first().click();
        await page.locator("[data-demo-expand]").first().click();
        await page.getByRole("button", { name: "PT", exact: true }).click();
        await page.screenshot({
          path: ".qa/pt-" + width + ".png",
          fullPage: true,
        });
        await page
          .getByRole("button", { name: "Alternar tema de cores" })
          .click();
        await page.screenshot({
          path: ".qa/dark-pt-" + width + ".png",
          fullPage: true,
        });
        const dark = await page.evaluate(async () =>
          (await axe.run()).violations.map((v) => ({
            id: v.id,
            impact: v.impact,
          })),
        );
        results.push({
          name: "dark-pt-" + width,
          overflow: await page.evaluate(
            () => document.documentElement.scrollWidth > innerWidth,
          ),
          violations: dark,
        });
        await page
          .getByRole("button", { name: "Debian / Ubuntu", exact: true })
          .click();
        if (!(await page.locator("pre").innerText()).includes("sudo apt"))
          throw Error("Install selection failed");
      }
      await page.close();
    }
  }
  await fs.writeFile(".qa/report.json", JSON.stringify(results, null, 2));
  console.log(JSON.stringify(results, null, 2));
} finally {
  await browser.close();
}
