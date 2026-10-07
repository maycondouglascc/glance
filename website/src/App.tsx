import { useState } from 'react'
import Wrapper from './components/Wrapper'
import SettingsBar from './components/SettingsBar'
import { TransitionChild } from './components/PageTransition'
import { GlanceInteractiveDemo } from './components/GlanceInteractiveDemo'
import { CopyButton } from './components/CopyButton'
import Footer from './components/Footer'
import { ThemeProvider } from './context/ThemeContext'
import { LanguageProvider, useLanguage } from './context/LanguageContext'

type InstallTab = 'curl' | 'deb' | 'aur' | 'cargo'

function MainContent() {
  const { t } = useLanguage()
  const [activeTab, setActiveTab] = useState<InstallTab>('curl')

  const installCommands: Record<InstallTab, { label: string; cmd: string; desc: string }> = {
    curl: {
      label: t('install.tabCurl'),
      cmd: 'curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sh',
      desc: 'Detects system arch, installs glance, glance-tree, and desktop entry to ~/.local/bin.',
    },
    deb: {
      label: t('install.tabDeb'),
      cmd: 'curl -LO https://github.com/maycondouglascc/glance/releases/latest/download/glance_0.1.0_amd64.deb && sudo apt install ./glance_0.1.0_amd64.deb',
      desc: 'Native package for Ubuntu 24.04+, Debian 13, Linux Mint, and derivatives.',
    },
    aur: {
      label: t('install.tabAur'),
      cmd: 'git clone https://github.com/maycondouglascc/glance.git && cd glance/packaging/aur && makepkg -si',
      desc: 'Arch Linux PKGBUILD build script for pacman / AUR helpers.',
    },
    cargo: {
      label: t('install.tabCargo'),
      cmd: 'cargo install --git https://github.com/maycondouglascc/glance.git glance',
      desc: 'Compiles and installs the latest binary using Rust toolchain.',
    },
  }

  const features = [
    {
      title: t('value.groupingTitle'),
      description: t('value.groupingDesc'),
    },
    {
      title: t('value.efficiencyTitle'),
      description: t('value.efficiencyDesc'),
    },
    {
      title: t('value.safetyTitle'),
      description: t('value.safetyDesc'),
    },
    {
      title: t('value.disclosureTitle'),
      description: t('value.disclosureDesc'),
    },
  ]

  return (
    <>
      <div className="fixed right-3 top-3 z-40 sm:right-5 sm:top-5">
        <SettingsBar />
      </div>

      <div className="px-1 py-1 sm:px-2 sm:py-2">
        <Wrapper>
          <div className="max-w-[600px] mx-auto">
            <main id="main-content" className="flex flex-col gap-12">
              
              {/* SECTION 0: HERO & INTERACTIVE APP */}
              <TransitionChild index={0}>
                <section className="flex flex-col gap-6">
                  {/* App Glyph & Version */}
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <span className="flex h-9 w-9 items-center justify-center rounded-lg bg-zinc-900 text-sm font-semibold text-white shadow-xs dark:bg-zinc-100 dark:text-zinc-900">
                        G
                      </span>
                      <div className="flex flex-col">
                        <span className="text-body-15-medium font-medium text-zinc-900 dark:text-zinc-100">
                          {t('intro.name')}
                        </span>
                        <span className="text-caption-12-regular text-zinc-500 dark:text-zinc-400">
                          Linux Process Monitor
                        </span>
                      </div>
                    </div>

                    <a
                      href="https://github.com/maycondouglascc/glance/releases/latest"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="rounded bg-zinc-100 px-2 py-0.5 font-mono text-caption-11-regular text-zinc-600 hover:text-zinc-900 dark:bg-zinc-800 dark:text-zinc-400 dark:hover:text-zinc-200 no-underline"
                    >
                      {t('intro.badge')}
                    </a>
                  </div>

                  {/* Title & Pitch */}
                  <div className="flex flex-col gap-2">
                    <h1 className="text-heading-28-medium font-semibold tracking-tight text-zinc-900 sm:text-heading-32-medium dark:text-zinc-100">
                      {t('intro.title')}
                    </h1>
                    <p className="text-body-15-regular font-normal text-zinc-600 dark:text-zinc-400">
                      {t('intro.description')}
                    </p>
                  </div>

                  {/* Quick 1-click terminal install pill */}
                  <div className="flex flex-col gap-1.5 rounded-xl bg-zinc-100/80 p-3 dark:bg-zinc-800/50 border border-zinc-200/60 dark:border-zinc-800/80">
                    <div className="flex items-center justify-between">
                      <span className="text-caption-11-regular font-mono text-zinc-500 dark:text-zinc-400">
                        {t('intro.quickInstall')}
                      </span>
                      <CopyButton text="curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sh" />
                    </div>
                    <code className="block overflow-x-auto font-mono text-caption-12-regular text-zinc-800 dark:text-zinc-200 select-all">
                      curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sh
                    </code>
                  </div>

                  {/* INTERACTIVE GLANCE DEMO */}
                  <div className="mt-2">
                    <GlanceInteractiveDemo />
                  </div>
                </section>
              </TransitionChild>

              {/* SECTION 1: VALUE PROPOSITIONS */}
              <TransitionChild index={1}>
                <section className="flex flex-col gap-4">
                  <div className="flex flex-col gap-1">
                    <h2 className="text-body-15-medium font-medium text-zinc-900 dark:text-zinc-100">
                      {t('value.title')}
                    </h2>
                    <p className="text-body-14-regular text-zinc-600 dark:text-zinc-400">
                      {t('value.subtitle')}
                    </p>
                  </div>

                  <div className="grid gap-4 sm:grid-cols-2 mt-2">
                    {features.map((feature, i) => (
                      <div
                        key={i}
                        className="flex flex-col gap-1.5 rounded-2xl bg-zinc-200/40 p-5 transition-colors duration-75 ease-in hover:bg-zinc-200/70 dark:bg-zinc-800/40 dark:hover:bg-zinc-800/70"
                      >
                        <h3 className="text-body-14-medium font-medium text-zinc-900 dark:text-zinc-100">
                          {feature.title}
                        </h3>
                        <p className="text-caption-13-regular text-zinc-600 dark:text-zinc-400">
                          {feature.description}
                        </p>
                      </div>
                    ))}
                  </div>
                </section>
              </TransitionChild>

              {/* SECTION 2: GET STARTED / INSTALLATION */}
              <TransitionChild index={2}>
                <section className="flex flex-col gap-4">
                  <div className="flex flex-col gap-1">
                    <h2 className="text-body-15-medium font-medium text-zinc-900 dark:text-zinc-100">
                      {t('install.title')}
                    </h2>
                    <p className="text-body-14-regular text-zinc-600 dark:text-zinc-400">
                      {t('install.subtitle')}
                    </p>
                  </div>

                  {/* Segmented radio tabs */}
                  <div className="mt-2 flex flex-wrap gap-1 rounded-lg bg-zinc-100 p-1 dark:bg-zinc-800/60 border border-zinc-200/60 dark:border-zinc-800/80">
                    {(Object.keys(installCommands) as InstallTab[]).map((tab) => (
                      <button
                        key={tab}
                        type="button"
                        onClick={() => setActiveTab(tab)}
                        className={`rounded-md px-3 py-1.5 text-caption-12-regular font-medium transition-colors duration-150 ${
                          activeTab === tab
                            ? 'bg-white text-zinc-900 shadow-xs dark:bg-zinc-800 dark:text-zinc-100'
                            : 'text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100'
                        }`}
                      >
                        {installCommands[tab].label}
                      </button>
                    ))}
                  </div>

                  {/* Command box */}
                  <div className="flex flex-col gap-2 rounded-2xl bg-zinc-200/40 p-5 dark:bg-zinc-800/40">
                    <div className="flex items-center justify-between">
                      <span className="text-caption-12-regular text-zinc-500 dark:text-zinc-400">
                        {installCommands[activeTab].desc}
                      </span>
                      <CopyButton text={installCommands[activeTab].cmd} />
                    </div>
                    <pre className="overflow-x-auto rounded bg-white p-3 font-mono text-caption-12-regular text-zinc-900 dark:bg-zinc-900 dark:text-zinc-100">
                      <code>{installCommands[activeTab].cmd}</code>
                    </pre>
                  </div>

                  {/* CLI Tip */}
                  <div className="rounded-xl border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-900 shadow-xs">
                    <p className="text-caption-13-regular text-zinc-600 dark:text-zinc-400">
                      <strong className="font-medium text-zinc-900 dark:text-zinc-100">
                        {t('install.cliHintTitle')}{' '}
                      </strong>
                      {t('install.cliHintText')}
                    </p>
                  </div>
                </section>
              </TransitionChild>

              {/* SECTION 3: ABOUT THE CREATOR */}
              <TransitionChild index={3}>
                <section className="flex flex-col gap-4">
                  <h2 className="text-body-15-medium font-medium text-zinc-900 dark:text-zinc-100">
                    {t('author.title')}
                  </h2>

                  <div className="flex items-start gap-4">
                    <img
                      src="./profilepic.webp"
                      alt="Maycon Douglas"
                      className="h-14 w-14 rounded-full object-cover ring-1 ring-zinc-200 dark:ring-zinc-800 shrink-0"
                    />
                    <div className="flex flex-col gap-2">
                      <p className="text-body-14-regular text-zinc-600 dark:text-zinc-400">
                        {t('author.bio')}
                      </p>
                      <a
                        href="https://maycondouglas.work"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-body-14-medium font-medium text-accent hover:underline dark:text-accent"
                      >
                        {t('author.portfolioLink')}
                      </a>
                    </div>
                  </div>
                </section>
              </TransitionChild>

            </main>

            {/* FOOTER */}
            <TransitionChild index={4}>
              <Footer />
            </TransitionChild>
          </div>
        </Wrapper>
      </div>
    </>
  )
}

export default function App() {
  return (
    <LanguageProvider>
      <ThemeProvider>
        <MainContent />
      </ThemeProvider>
    </LanguageProvider>
  )
}
