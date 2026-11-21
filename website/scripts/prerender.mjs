import { launch } from "./browser.mjs";
import { preview } from "vite";
import { readFile, writeFile } from "node:fs/promises";

// Static HTML keeps the landing content available before React and to crawlers.
const production = await preview({
  preview: { host: "127.0.0.1", port: 4499, strictPort: true },
});
let browser;
try {
  browser = await launch();
  const page = await browser.newPage({
    reducedMotion: "reduce",
    locale: "en-US",
  });
  // Analytics are not needed during the build.
  await page.route("https://**/*", (route) => route.abort());
  const original = await readFile("dist/index.html", "utf8");
  for (const [query, output] of [
    ["", "index.html"],
    ["?page=guide", "guide.html"],
    ["?page=404", "404.html"],
  ]) {
    await page.goto("http://127.0.0.1:4499/" + query);
    await page.waitForSelector("h1");
    await page.evaluate(() => document.fonts.ready);
    const root = await page.locator("#root").innerHTML();
    const title = await page.title();
    const description = await page
      .locator('meta[name="description"]')
      .getAttribute("content");
    const canonical = await page
      .locator('link[rel="canonical"]')
      .getAttribute("href");
    const escape = (value) =>
      value.replaceAll("&", "&amp;").replaceAll('"', "&quot;");
    const html = original
      .replace('<div id="root"></div>', `<div id="root">${root}</div>`)
      .replace(/<title>.*?<\/title>/, `<title>${title}</title>`)
      .replace(
        /(<meta name="description" content=")[^"]*/,
        "$1" + escape(description),
      )
      .replace(/(<link rel="canonical" href=")[^"]*/, "$1" + canonical);
    await writeFile("dist/" + output, html);
  }
  await page.setViewportSize({ width: 1200, height: 630 });
  await page.goto("http://127.0.0.1:4499/");
  await page.waitForSelector("h1");
  await page.evaluate(() => document.fonts.ready);
  await page.addStyleTag({
    content:
      ".header{height:80px}.hero{padding:55px 0}.hero h1{font-size:84px}.site-shell{padding:0 48px}.facts,footer{display:none}",
  });
  await page.screenshot({ path: "dist/og.png" });
  console.log("Prerendered home, guide and 404; generated OG image.");
} finally {
  await browser?.close();
  production.httpServer.close();
}
