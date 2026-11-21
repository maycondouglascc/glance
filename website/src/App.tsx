import { useEffect, useRef, useState } from "react";
import { LanguageProvider, useLanguage } from "./context/LanguageContext";
import { ThemeProvider } from "./context/ThemeContext";
import { copy, methods, site } from "./data/site";
import { GlanceInteractiveDemo } from "./components/GlanceInteractiveDemo";
import { usePageMotion } from "./motion/usePageMotion";

function Arrow() {
  return (
    <span className="arrow" aria-hidden="true">
      ↗
    </span>
  );
}
function DownloadIcon() {
  return (
    <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 3v12m-4-4 4 4 4-4M4 17v3h16v-3" />
    </svg>
  );
}
function Installation({ guide = false }: { guide?: boolean }) {
  const { language } = useLanguage();
  const c = copy[language];
  const [method, setMethod] = useState(0);
  const [status, setStatus] = useState<"idle" | "copied" | "failed">("idle");
  const timer = useRef<ReturnType<typeof setTimeout>>();
  useEffect(() => () => clearTimeout(timer.current), []);
  async function copyCommand() {
    clearTimeout(timer.current);
    try {
      await navigator.clipboard.writeText(methods[method]);
      setStatus("copied");
    } catch {
      setStatus("failed");
    }
    timer.current = setTimeout(() => setStatus("idle"), 3000);
  }
  return (
    <section
      className="installation section-grid"
      id="install"
      aria-labelledby="install-title"
    >
      <div className="install-intro">
        <h2 id="install-title">{c.installTitle}</h2>
        <p>{c.installText}</p>
      </div>
      <div className="install-workbench">
        <div className="install-methods" role="group" aria-label={c.guide}>
          {c.methods.map((label, i) => (
            <button
              key={label}
              aria-pressed={method === i}
              onClick={() => {
                setMethod(i);
                setStatus("idle");
                clearTimeout(timer.current);
              }}
            >
              {label}
            </button>
          ))}
        </div>
        <div className="command-box">
          <div className="command-header">
            <span>~/glance</span>
            <button onClick={copyCommand}>
              {status === "copied" ? "✓ " + c.copied : c.copy}
              <span aria-hidden="true"> ⧉</span>
            </button>
          </div>
          <pre tabIndex={0}>
            <code>{methods[method]}</code>
          </pre>
        </div>
        {guide && <p className="command-description">
          {c.descriptions[method]}{" "}
          {method === 1 && <a href={site.releases}>GitHub Releases ↗</a>}
        </p>}
        <p role="status" className="copy-status">
          {status === "copied" ? c.copied : status === "failed" ? c.failed : ""}
        </p>
        {guide && <><p className="launcher-tip">
          <span className="keycap">Super</span>
          {c.launcher}
        </p>
        <div className="terminal-tip">
          <strong>{c.terminal}</strong>
          <p>{c.terminalText}</p>
        </div></>}
      </div>
    </section>
  );
}
function MainContent() {
  const { language } = useLanguage();
  const c = copy[language];
  const query = new URLSearchParams(window.location.search).get("page");
  const path = window.location.pathname;
  const page =
    query === "guide" || path.endsWith("/guide.html")
      ? "guide"
      : query ||
        (!path.endsWith("/") &&
        !path.endsWith("/index.html") &&
        !path.endsWith("/glance")
          ? "404"
          : "home");
  usePageMotion(page + language);
  useEffect(() => {
    document.documentElement.lang = language === "pt" ? "pt-BR" : "en";
    document.title =
      page === "home"
        ? "Glance | " + c.label
        : (page === "guide" ? c.guide : "404") + " | Glance";
    document
      .querySelector('meta[name="description"]')
      ?.setAttribute(
        "content",
        page === "guide"
          ? c.guideText
          : page === "home"
            ? c.description
            : c.notFoundText,
      );
    document
      .querySelector('link[rel="canonical"]')
      ?.setAttribute(
        "href",
        site.url + (page === "guide" ? "?page=guide" : ""),
      );
  }, [language, page, c]);
  return (
    <>
      <a className="skip-link" href="#main-content">
        {c.skip}
      </a>
      <div className="site-shell">
        <header className="header">
          <a
            className="wordmark"
            href={import.meta.env.BASE_URL}
            aria-label="Glance home"
          >
            <img
              src={import.meta.env.BASE_URL + "logo.svg"}
              alt=""
              width="36"
              height="36"
            />
            Glance<span className="brand-dot">.</span>
          </a>
        </header>
        <main id="main-content" tabIndex={-1}>
          {page === "home" ? (
            <>
              <section className="hero">
                <div className="hero-copy">
                  <h1 data-intro>
                    {c.headline[0]}
                    <br />
                    <span>{c.headline[1]}</span>
                  </h1>
                  <p className="hero-description" data-intro>
                    {c.description}
                  </p>
                  <div className="hero-action" data-intro>
                    <a className="primary-button" href="#install">
                      <img src={import.meta.env.BASE_URL + "icons/linux.svg"} alt="" width="20" height="20" />
                      {c.cta}
                      <DownloadIcon />
                    </a>
                  </div>
                </div>
                <div className="hero-demo" id="demo" data-intro>
                  <GlanceInteractiveDemo />
                </div>
              </section>
              <section
                className="facts"
                aria-label={
                  language === "pt"
                    ? "Características técnicas"
                    : "Technical highlights"
                }
              >
                <div>
                  <strong>
                    0<span>%</span>
                  </strong>
                  <p>{c.idle}</p>
                </div>
                <div>
                  <strong>
                    <span>~</span>65 <span>MB</span>
                  </strong>
                  <p>{c.memory}</p>
                </div>
                <div>
                  <strong className="native-fact">
                    GTK4 <span>+ Rust</span>
                  </strong>
                  <p>{c.native}</p>
                </div>
              </section>
              <section className="explore section-grid" id="explore">
                <h2 className="explore-title">{c.why}</h2>
                <div className="explore-content">
                  <p className="section-description">{c.whyText}</p>
                  <div className="feature-index">
                  {c.features.map(([title, description], i) => (
                    <article key={title}>
                      <span className="feature-symbol" aria-hidden="true">
                        {["⊞", "⊙", "⌁"][i]}
                      </span>
                      <div>
                        <h3>{title}</h3>
                        <p>{description}</p>
                      </div>
                    </article>
                  ))}
                  </div>
                </div>
              </section>
              <Installation />
            </>
          ) : page === "guide" ? (
            <>
              <section className="inner-hero">
                <p>Glance / {c.guide}</p>
                <h1>{c.guideTitle}</h1>
                <p>{c.guideText}</p>
              </section>
              <Installation guide />
            </>
          ) : (
            <section className="not-found">
              <span className="error-code">404</span>
              <h1>{c.notFound}</h1>
              <p>{c.notFoundText}</p>
              <a className="primary-button" href={import.meta.env.BASE_URL}>
                {c.home}
                <Arrow />
              </a>
            </section>
          )}
        </main>
        <footer>
          <div className="footer-top">
            <p>{c.footer}</p>
            <a className="text-link" href={site.github}>
              GitHub
              <Arrow />
            </a>
          </div>
          <div className="footer-wordmark" aria-hidden="true">
            <img src={import.meta.env.BASE_URL + "logo.svg"} alt="" width="56" height="56" />
          </div>
        </footer>
      </div>
    </>
  );
}
export default function App() {
  return (
    <LanguageProvider>
      <ThemeProvider>
        <MainContent />
      </ThemeProvider>
    </LanguageProvider>
  );
}
