export const en = {
  settings: {
    label: 'Settings',
  },
  language: {
    selectorLabel: 'Select language',
    english: 'EN',
    portuguese: 'PT',
    englishLong: 'English',
    portugueseLong: 'Português',
  },
  theme: {
    toggle: 'Toggle theme',
    light: 'Light',
    dark: 'Dark',
    system: 'System',
  },
  intro: {
    name: 'Glance',
    badge: 'v0.1.0',
    title: 'Grouped process monitor for Linux',
    description:
      'A small, fast Linux utility that groups running processes by application with progressive disclosure. 0% idle CPU and ~65 MB RAM in the system tray.',
    quickInstall: 'Quick install to ~/.local/bin',
    copy: 'Copy',
    copied: 'Copied!',
  },
  demo: {
    badge: 'Interactive Interface',
    hint: 'Click chevrons to expand, search by app name or PID, or click (i) to inspect.',
    searchPlaceholder: 'Search apps or PID...',
    colApp: 'APPLICATION',
    colCpu: 'CPU',
    colRam: 'RAM',
    secApp: 'APPLICATION',
    secBg: 'BACKGROUND',
    secSys: 'SYSTEM',
    procs: '{count} procs',
    inspect: 'Inspect details',
    terminate: 'End process (SIGTERM)',
    reset: 'Reset demo state',
    status: 'CPU {cpu}% · RAM {ram} GB · {procs} processes',
    detailsTitle: 'Process Telemetry',
    pid: 'PID',
    ppid: 'PPID',
    state: 'State',
    threads: 'Threads',
    cmdline: 'Command Line',
    close: 'Close',
  },
  value: {
    title: 'Why use Glance',
    subtitle:
      'Standard system monitors overwhelm you with 80 disconnected rows for a single browser. Glance brings structure without sacrificing performance.',
    groupingTitle: 'Application Grouping',
    groupingDesc:
      'Related processes automatically gather under their parent application using systemd cgroup scopes and desktop entries.',
    efficiencyTitle: '0.00% Idle CPU',
    efficiencyDesc:
      'Drops to true zero CPU when minimized or running in the tray via blocking kernel event loops. Uses ~65 MB RSS with mimalloc and Cairo 2D acceleration.',
    safetyTitle: 'Safe Process Signals',
    safetyDesc:
      'Utilizes Linux pidfd and start-time validation to eliminate PID-reuse hazards. Never accidentally terminates the wrong process.',
    disclosureTitle: 'Progressive Disclosure',
    disclosureDesc:
      'Clean top-level summaries by default. Double-click or expand only when you need to inspect renderers, extensions, or background workers.',
  },
  install: {
    title: 'Get Started',
    subtitle: 'Available for all modern Linux distributions. Choose your preferred method:',
    tabCurl: 'Universal Script',
    tabDeb: 'Ubuntu / Debian',
    tabAur: 'Arch Linux',
    tabCargo: 'Cargo',
    cliHintTitle: 'CLI Companion:',
    cliHintText:
      'Glance includes glance-tree for terminal power users. Run glance-tree -c for an expanded process tree in your terminal.',
  },
  author: {
    title: 'About the creator',
    bio:
      'Designed and developed by Maycon Douglas, Product Designer. Glance was created to solve everyday friction with cluttered Linux process lists, focusing on software craft, performance, and clean human interface design.',
    portfolioLink: 'Explore my design portfolio →',
  },
  footer: {
    madeWith: 'Made with <3',
    license: 'GPL-3.0 License',
  },
}
