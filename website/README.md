# Glance website

React + Vite, with the design direction in ../DESIGN.md. Content and commands live in src/data/site.ts. The site is in English and follows the system color theme. Demo telemetry is illustrative.

```sh
npm ci
npx playwright install chromium
npm run dev
```

Production build and preview:

```sh
npm run typecheck
npm run build
npm run preview -- --host 127.0.0.1 --port 4400
```

Open http://127.0.0.1:4400/. Build includes a prerendered home, guide.html, 404.html and a generated 1200×630 og.png. The guide also works at ?page=guide. Unknown URLs receive HTTP 404 in the preview server; a static host must use the generated 404.html as its not-found page.

The build uses the domain root and the canonical URL https://glance.maycondouglas.work/.

The production site runs on Cloudflare Workers static assets. After authenticating with `cf auth login`, run `npm run build` and `python3 scripts/deploy-cloudflare.py` from this directory to publish the generated files. The custom domain is configured on the `glance-landing` Worker. The redirect from the old URL is configured in the portfolio project's `vercel.json`.

Runtime browsers are needed for building the static HTML and browser verification.
