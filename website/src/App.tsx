import { useState } from 'react'
import { motion } from 'motion/react'
import { ThemeToggle } from './components/ThemeToggle'
import { CopyButton } from './components/CopyButton'

type InstallTab = 'curl' | 'deb' | 'aur' | 'cargo'

const INSTALL_OPTIONS: Record<InstallTab, { label: string; cmd: string; desc: string }> = {
  curl: {
    label: 'Universal Script',
    cmd: 'curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sh',
    desc: 'Installs glance, glance-tree, and desktop launcher to ~/.local/bin. No root required.',
  },
  deb: {
    label: 'Ubuntu / Debian',
    cmd: 'curl -LO https://github.com/maycondouglascc/glance/releases/latest/download/glance_0.1.0_amd64.deb && sudo apt install ./glance_0.1.0_amd64.deb',
    desc: 'Official native Debian package for Ubuntu 24.04+, Debian 13, and derivatives.',
  },
  aur: {
    label: 'Arch Linux',
    cmd: 'git clone https://github.com/maycondouglascc/glance.git && cd glance/packaging/aur && makepkg -si',
    desc: 'Native package built with makepkg from source / release binaries.',
  },
  cargo: {
    label: 'Cargo',
    cmd: 'cargo install --git https://github.com/maycondouglascc/glance.git glance',
    desc: 'Compiles and installs directly from the latest git commit using Rust toolchain.',
  },
}

const HIGHLIGHTS = [
  {
    title: 'Grouped by Application',
    description:
      'Related processes automatically gather under their parent app using systemd cgroups. See one clean row for Firefox or VS Code instead of 60 noisy PID rows.',
  },
  {
    title: '0.00% Idle CPU',
    description:
      'Kernel-blocking event loops pause when minimized or closed to the system tray. Uses ~65 MB RAM with mimalloc and Cairo 2D acceleration.',
  },
  {
    title: 'Safe Process Signals',
    description:
      'Uses Linux pidfd and start-time validation to eliminate PID-reuse race conditions. You will never accidentally kill the wrong process.',
  },
  {
    title: 'Type-to-Search & Shortcuts',
    description:
      'Type anywhere for instantaneous filtering by name, command line, or PID. Delete gracefully closes an app (SIGTERM); Shift+Delete force kills.',
  },
]

export default function App() {
  const [activeTab, setActiveTab] = useState<InstallTab>('curl')

  return (
    <div className="min-h-screen px-3 py-6 sm:px-6 sm:py-12">
      {/* Outer container matching Maycon's portfolio Wrapper */}
      <div className="relative mx-auto w-full max-w-[640px] rounded-2xl bg-white p-6 shadow-sm transition-all sm:p-10 dark:bg-zinc-900 dark:shadow-none dark:ring-1 dark:ring-zinc-800">
        
        {/* Top Header */}
        <header className="flex items-center justify-between border-b border-zinc-200 pb-6 dark:border-zinc-800">
          <div className="flex items-center gap-3">
            <span className="flex h-7 w-7 items-center justify-center rounded-lg bg-zinc-900 text-xs font-semibold text-white dark:bg-zinc-100 dark:text-zinc-900">
              G
            </span>
            <div className="flex items-center gap-2">
              <span className="text-body-15-medium font-medium text-zinc-900 dark:text-zinc-100">
                Glance
              </span>
              <span className="rounded bg-zinc-100 px-1.5 py-0.5 text-caption-11-regular font-mono text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400">
                v0.1.0
              </span>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <a
              href="https://github.com/maycondouglascc/glance"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex h-8 items-center gap-1.5 rounded-md border border-zinc-200 px-2.5 text-caption-12-medium text-zinc-600 no-underline transition-colors hover:bg-zinc-100 hover:text-zinc-900 hover:no-underline dark:border-zinc-800 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-100"
            >
              <svg className="h-3.5 w-3.5" viewBox="0 0 24 24" fill="currentColor">
                <path fillRule="evenodd" clipRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
              </svg>
              <span>GitHub</span>
            </a>
            <ThemeToggle />
          </div>
        </header>

        {/* Hero Section */}
        <section className="mt-8 flex flex-col gap-5">
          <div>
            <h1 className="text-heading-32-medium font-semibold tracking-tight text-zinc-900 sm:text-heading-40-medium dark:text-zinc-100">
              Glance.
            </h1>
            <p className="mt-2 text-body-15-medium text-zinc-600 dark:text-zinc-400">
              A lightweight, high-performance Linux desktop app that groups running processes by application with progressive disclosure.
            </p>
          </div>

          {/* Quick 1-line curl box */}
          <div className="flex flex-col gap-2 rounded-xl border border-zinc-200 bg-zinc-50/80 p-3.5 dark:border-zinc-800 dark:bg-zinc-950/60">
            <div className="flex items-center justify-between text-caption-12-medium text-zinc-500 dark:text-zinc-400">
              <span>Quick install</span>
              <CopyButton text="curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sh" />
            </div>
            <code className="block overflow-x-auto font-mono text-caption-12-regular text-zinc-800 dark:text-zinc-200">
              curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sh
            </code>
          </div>

          {/* Hero App Image */}
          <div className="mt-2 overflow-hidden rounded-xl border border-zinc-200 bg-zinc-950 shadow-sm dark:border-zinc-800">
            <div className="flex items-center justify-between border-b border-zinc-800/80 bg-zinc-900/90 px-3.5 py-2 text-caption-11-regular font-mono text-zinc-400">
              <span className="flex items-center gap-1.5">
                <span className="h-2 w-2 rounded-full bg-zinc-700" />
                <span>glance</span>
              </span>
              <span>x86_64 · Linux</span>
            </div>
            <div className="flex items-center justify-center bg-zinc-950 p-2 sm:p-4">
              <img
                src="./glance-app.png"
                alt="Glance Linux process monitor user interface"
                className="max-h-[520px] w-auto rounded-lg object-contain shadow-2xl"
                loading="eager"
              />
            </div>
          </div>
        </section>

        {/* Why Glance / Value Props */}
        <section className="mt-12 border-t border-zinc-200 pt-8 dark:border-zinc-800">
          <h2 className="text-subheading-20-medium font-semibold text-zinc-900 dark:text-zinc-100">
            Why use Glance
          </h2>
          <p className="mt-1 text-body-14-regular text-zinc-600 dark:text-zinc-400">
            Traditional task managers display flat tables with hundreds of disconnected PIDs. Glance is built for clarity and efficiency.
          </p>

          <div className="mt-6 grid gap-4 sm:grid-cols-2">
            {HIGHLIGHTS.map((item, index) => (
              <div
                key={index}
                className="flex flex-col gap-1.5 rounded-xl border border-zinc-200 bg-zinc-50/50 p-4 transition-colors hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-950/40 dark:hover:border-zinc-700"
              >
                <h3 className="text-body-14-medium font-medium text-zinc-900 dark:text-zinc-100">
                  {item.title}
                </h3>
                <p className="text-caption-13-regular text-zinc-600 dark:text-zinc-400">
                  {item.description}
                </p>
              </div>
            ))}
          </div>
        </section>

        {/* Getting Started / Installation Guide */}
        <section className="mt-12 border-t border-zinc-200 pt-8 dark:border-zinc-800">
          <div className="flex items-center justify-between">
            <h2 className="text-subheading-20-medium font-semibold text-zinc-900 dark:text-zinc-100">
              Get Started
            </h2>
            <span className="text-caption-12-medium text-zinc-500 dark:text-zinc-400">
              Linux only
            </span>
          </div>
          <p className="mt-1 text-body-14-regular text-zinc-600 dark:text-zinc-400">
            Choose your preferred method to install Glance:
          </p>

          {/* Segmented control tabs */}
          <div className="mt-4 flex flex-wrap gap-1.5 rounded-lg border border-zinc-200 bg-zinc-100 p-1 dark:border-zinc-800 dark:bg-zinc-950">
            {(Object.keys(INSTALL_OPTIONS) as InstallTab[]).map((tab) => (
              <button
                key={tab}
                type="button"
                onClick={() => setActiveTab(tab)}
                className={`rounded-md px-3 py-1.5 text-caption-12-medium transition-all ${
                  activeTab === tab
                    ? 'bg-white font-medium text-zinc-900 shadow-xs dark:bg-zinc-800 dark:text-zinc-100'
                    : 'text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-200'
                }`}
              >
                {INSTALL_OPTIONS[tab].label}
              </button>
            ))}
          </div>

          {/* Command display block */}
          <div className="mt-3 flex flex-col gap-2 rounded-xl border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-950/70">
            <div className="flex items-center justify-between">
              <span className="text-caption-12-regular text-zinc-500 dark:text-zinc-400">
                {INSTALL_OPTIONS[activeTab].desc}
              </span>
              <CopyButton text={INSTALL_OPTIONS[activeTab].cmd} />
            </div>
            <pre className="overflow-x-auto rounded font-mono text-caption-12-regular text-zinc-800 dark:text-zinc-200">
              <code>{INSTALL_OPTIONS[activeTab].cmd}</code>
            </pre>
          </div>

          {/* CLI companion tip */}
          <div className="mt-4 flex items-start gap-3 rounded-lg border border-zinc-200 bg-white p-3.5 dark:border-zinc-800 dark:bg-zinc-900">
            <span className="mt-0.5 text-base">⚡</span>
            <div className="text-caption-13-regular text-zinc-600 dark:text-zinc-400">
              <strong className="font-medium text-zinc-900 dark:text-zinc-100">CLI Companion:</strong> Glance also includes <code className="font-mono text-caption-12-regular text-zinc-800 dark:text-zinc-200">glance-tree</code> for instantaneous process tree viewing directly inside your terminal (<code className="font-mono text-caption-12-regular">glance-tree -c</code>).
            </div>
          </div>
        </section>

        {/* Creator & Portfolio link */}
        <footer className="mt-12 border-t border-zinc-200 pt-6 dark:border-zinc-800">
          <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div className="text-body-14-regular text-zinc-600 dark:text-zinc-400">
              Crafted by{' '}
              <a
                href="https://maycondouglas.work"
                target="_blank"
                rel="noopener noreferrer"
                className="font-medium text-zinc-900 hover:underline dark:text-zinc-100"
              >
                Maycon Douglas
              </a>
              . Open source under GPL-3.0.
            </div>

            <div className="flex items-center gap-4 text-caption-13-medium">
              <a
                href="https://maycondouglas.work"
                target="_blank"
                rel="noopener noreferrer"
                className="text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100"
              >
                Portfolio
              </a>
              <a
                href="https://github.com/maycondouglascc/glance"
                target="_blank"
                rel="noopener noreferrer"
                className="text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100"
              >
                Source Code
              </a>
              <a
                href="https://github.com/maycondouglascc"
                target="_blank"
                rel="noopener noreferrer"
                className="text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100"
              >
                GitHub
              </a>
            </div>
          </div>
        </footer>

      </div>
    </div>
  )
}
