import { useState, useMemo } from 'react'
import { motion, AnimatePresence } from 'motion/react'
import { useLanguage } from '../context/LanguageContext'

interface SubProcess {
  pid: number
  name: string
  role: string
  cmdline: string
  exe: string
  cpu: number
  memMb: number
  pssMb: number
  threads: number
  uid: number
  state: string
}

interface AppGroup {
  id: string
  name: string
  iconType: 'firefox' | 'vscode' | 'spotify' | 'terminal'
  processes: SubProcess[]
}

interface FlatProcess {
  pid: number
  name: string
  cmdline: string
  exe: string
  cpu: number
  memMb: number
  pssMb: number
  threads: number
  uid: number
  state: string
}

const INITIAL_APPS: AppGroup[] = [
  {
    id: 'firefox',
    name: 'Firefox',
    iconType: 'firefox',
    processes: [
      {
        pid: 19220,
        name: 'firefox',
        role: 'Main UI Process',
        cmdline: '/usr/lib/firefox/firefox --new-window',
        exe: '/usr/lib/firefox/firefox',
        cpu: 0.8,
        memMb: 340,
        pssMb: 290,
        threads: 64,
        uid: 1000,
        state: 'Running (R)',
      },
      {
        pid: 19285,
        name: 'Web Content',
        role: 'Tab: GitHub · glance',
        cmdline: '/usr/lib/firefox/firefox -contentproc -childID 1',
        exe: '/usr/lib/firefox/firefox',
        cpu: 1.9,
        memMb: 410,
        pssMb: 380,
        threads: 28,
        uid: 1000,
        state: 'Sleeping (S)',
      },
      {
        pid: 19310,
        name: 'Web Content',
        role: 'Tab: Rust Documentation',
        cmdline: '/usr/lib/firefox/firefox -contentproc -childID 2',
        exe: '/usr/lib/firefox/firefox',
        cpu: 0.9,
        memMb: 270,
        pssMb: 240,
        threads: 24,
        uid: 1000,
        state: 'Sleeping (S)',
      },
      {
        pid: 19342,
        name: 'RDD Process',
        role: 'Media Decoder',
        cmdline: '/usr/lib/firefox/firefox -contentproc -rdd',
        exe: '/usr/lib/firefox/firefox',
        cpu: 0.4,
        memMb: 110,
        pssMb: 95,
        threads: 16,
        uid: 1000,
        state: 'Sleeping (S)',
      },
      {
        pid: 19355,
        name: 'Socket Process',
        role: 'Network I/O',
        cmdline: '/usr/lib/firefox/firefox -contentproc -socket',
        exe: '/usr/lib/firefox/firefox',
        cpu: 0.2,
        memMb: 50,
        pssMb: 42,
        threads: 12,
        uid: 1000,
        state: 'Sleeping (S)',
      },
    ],
  },
  {
    id: 'vscode',
    name: 'Visual Studio Code',
    iconType: 'vscode',
    processes: [
      {
        pid: 24102,
        name: 'code',
        role: 'Main Window (Electron)',
        cmdline: '/usr/share/code/code . --unity-launch',
        exe: '/usr/share/code/code',
        cpu: 0.9,
        memMb: 280,
        pssMb: 250,
        threads: 32,
        uid: 1000,
        state: 'Running (R)',
      },
      {
        pid: 24150,
        name: 'extension-host',
        role: 'Rust Analyzer Daemon',
        cmdline: '/usr/share/code/code --type=extensionHost --pid=24102',
        exe: '/usr/share/code/code',
        cpu: 1.4,
        memMb: 360,
        pssMb: 320,
        threads: 18,
        uid: 1000,
        state: 'Running (R)',
      },
      {
        pid: 24205,
        name: 'pty-host',
        role: 'Integrated Terminal PTY',
        cmdline: '/usr/share/code/code --type=ptyHost',
        exe: '/usr/share/code/code',
        cpu: 0.5,
        memMb: 140,
        pssMb: 120,
        threads: 8,
        uid: 1000,
        state: 'Sleeping (S)',
      },
    ],
  },
  {
    id: 'spotify',
    name: 'Spotify',
    iconType: 'spotify',
    processes: [
      {
        pid: 14201,
        name: 'spotify',
        role: 'Client Interface',
        cmdline: '/opt/spotify/spotify',
        exe: '/opt/spotify/spotify',
        cpu: 0.3,
        memMb: 170,
        pssMb: 150,
        threads: 24,
        uid: 1000,
        state: 'Sleeping (S)',
      },
      {
        pid: 14210,
        name: 'spotify-audio',
        role: 'Audio Engine & Decoder',
        cmdline: '/opt/spotify/spotify --audio-worker',
        exe: '/opt/spotify/spotify',
        cpu: 0.4,
        memMb: 120,
        pssMb: 105,
        threads: 14,
        uid: 1000,
        state: 'Running (R)',
      },
    ],
  },
  {
    id: 'terminal',
    name: 'Terminal',
    iconType: 'terminal',
    processes: [
      {
        pid: 8812,
        name: 'alacritty',
        role: 'Terminal Window',
        cmdline: '/usr/bin/alacritty',
        exe: '/usr/bin/alacritty',
        cpu: 0.1,
        memMb: 32,
        pssMb: 28,
        threads: 6,
        uid: 1000,
        state: 'Sleeping (S)',
      },
      {
        pid: 8820,
        name: 'zsh',
        role: 'Shell Session (glance-tree)',
        cmdline: '/bin/zsh -i',
        exe: '/bin/zsh',
        cpu: 0.1,
        memMb: 16,
        pssMb: 14,
        threads: 1,
        uid: 1000,
        state: 'Sleeping (S)',
      },
    ],
  },
]

const INITIAL_BG_PROCESSES: FlatProcess[] = [
  {
    pid: 1280,
    name: 'dockerd',
    cmdline: '/usr/bin/dockerd -H fd://',
    exe: '/usr/bin/dockerd',
    cpu: 0.3,
    memMb: 190,
    pssMb: 160,
    threads: 22,
    uid: 0,
    state: 'Sleeping (S)',
  },
  {
    pid: 3410,
    name: '1password-helper',
    cmdline: '/opt/1Password/1password --silent',
    exe: '/opt/1Password/1password',
    cpu: 0.1,
    memMb: 85,
    pssMb: 70,
    threads: 10,
    uid: 1000,
    state: 'Sleeping (S)',
  },
  {
    pid: 912,
    name: 'tailscaled',
    cmdline: '/usr/sbin/tailscaled --state=/var/lib/tailscale/tailscaled.state',
    exe: '/usr/sbin/tailscaled',
    cpu: 0.0,
    memMb: 34,
    pssMb: 28,
    threads: 8,
    uid: 0,
    state: 'Sleeping (S)',
  },
]

const INITIAL_SYS_PROCESSES: FlatProcess[] = [
  {
    pid: 410,
    name: 'systemd-journald',
    cmdline: '/usr/lib/systemd/systemd-journald',
    exe: '/usr/lib/systemd/systemd-journald',
    cpu: 0.0,
    memMb: 22,
    pssMb: 18,
    threads: 1,
    uid: 0,
    state: 'Sleeping (S)',
  },
  {
    pid: 1120,
    name: 'pipewire-pulse',
    cmdline: '/usr/bin/pipewire-pulse',
    exe: '/usr/bin/pipewire-pulse',
    cpu: 0.2,
    memMb: 28,
    pssMb: 24,
    threads: 2,
    uid: 1000,
    state: 'Running (R)',
  },
]

function AppIcon({ type }: { type: AppGroup['iconType'] }) {
  switch (type) {
    case 'firefox':
      return (
        <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded-md bg-gradient-to-tr from-amber-500 via-orange-500 to-red-500 text-[10px] shadow-xs">
          🦊
        </span>
      )
    case 'vscode':
      return (
        <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded-md bg-sky-600 text-[10px] text-white shadow-xs">
          💻
        </span>
      )
    case 'spotify':
      return (
        <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded-md bg-emerald-600 text-[10px] text-white shadow-xs">
          🎵
        </span>
      )
    case 'terminal':
      return (
        <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded-md bg-zinc-800 text-[10px] text-zinc-300 ring-1 ring-zinc-700">
          $
        </span>
      )
  }
}

export function GlanceInteractiveDemo() {
  const { t } = useLanguage()

  // State
  const [query, setQuery] = useState('')
  const [expandedApps, setExpandedApps] = useState<Set<string>>(new Set(['firefox']))
  const [expandBg, setExpandBg] = useState(false)
  const [expandSys, setExpandSys] = useState(false)
  const [killedPids, setKilledPids] = useState<Set<number>>(new Set())
  const [selectedProcess, setSelectedProcess] = useState<SubProcess | FlatProcess | null>(null)

  // Toggle app accordion
  const toggleApp = (id: string) => {
    setExpandedApps((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  // Terminate a single PID
  const killProcess = (pid: number, e?: React.MouseEvent) => {
    e?.stopPropagation()
    setKilledPids((prev) => new Set(prev).add(pid))
    if (selectedProcess?.pid === pid) {
      setSelectedProcess(null)
    }
  }

  // Terminate an entire application
  const killApp = (app: AppGroup, e: React.MouseEvent) => {
    e.stopPropagation()
    setKilledPids((prev) => {
      const next = new Set(prev)
      app.processes.forEach((p) => next.add(p.pid))
      return next
    })
  }

  // Reset demo
  const resetDemo = () => {
    setKilledPids(new Set())
    setExpandedApps(new Set(['firefox']))
    setQuery('')
    setSelectedProcess(null)
  }

  // Filtered live data
  const q = query.trim().toLowerCase()

  const liveApps = useMemo(() => {
    return INITIAL_APPS.map((app) => {
      const aliveProcs = app.processes.filter((p) => !killedPids.has(p.pid))
      const matchingProcs = q
        ? aliveProcs.filter(
            (p) =>
              p.name.toLowerCase().includes(q) ||
              p.role.toLowerCase().includes(q) ||
              p.cmdline.toLowerCase().includes(q) ||
              String(p.pid).includes(q) ||
              app.name.toLowerCase().includes(q)
          )
        : aliveProcs

      const totalCpu = matchingProcs.reduce((acc, p) => acc + p.cpu, 0)
      const totalMemMb = matchingProcs.reduce((acc, p) => acc + p.memMb, 0)

      return {
        ...app,
        aliveCount: aliveProcs.length,
        matchingProcs,
        totalCpu,
        totalMemMb,
        isVisible: matchingProcs.length > 0,
      }
    }).filter((app) => app.isVisible)
  }, [killedPids, q])

  const liveBg = useMemo(() => {
    const alive = INITIAL_BG_PROCESSES.filter((p) => !killedPids.has(p.pid))
    if (!q) return alive
    return alive.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        p.cmdline.toLowerCase().includes(q) ||
        String(p.pid).includes(q)
    )
  }, [killedPids, q])

  const liveSys = useMemo(() => {
    const alive = INITIAL_SYS_PROCESSES.filter((p) => !killedPids.has(p.pid))
    if (!q) return alive
    return alive.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        p.cmdline.toLowerCase().includes(q) ||
        String(p.pid).includes(q)
    )
  }, [killedPids, q])

  // Total summary calculations
  const totalStats = useMemo(() => {
    let cpu = 0
    let memMb = 0
    let count = 0

    liveApps.forEach((a) => {
      cpu += a.totalCpu
      memMb += a.totalMemMb
      count += a.matchingProcs.length
    })

    liveBg.forEach((p) => {
      cpu += p.cpu
      memMb += p.memMb
      count += 1
    })

    liveSys.forEach((p) => {
      cpu += p.cpu
      memMb += p.memMb
      count += 1
    })

    return {
      cpu: cpu.toFixed(1),
      ram: (memMb / 1024).toFixed(2),
      procs: count,
    }
  }, [liveApps, liveBg, liveSys])

  const hasKilled = killedPids.size > 0

  return (
    <div className="relative mt-2 overflow-hidden rounded-xl border border-zinc-200 bg-zinc-950 font-sans shadow-lg dark:border-zinc-800">
      
      {/* Adwaita Header Bar */}
      <div className="flex items-center gap-2 border-b border-zinc-800/80 bg-zinc-900/95 px-3 py-2.5">
        <button
          type="button"
          className="flex h-7 w-7 items-center justify-center rounded text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-200"
          title="Filter sort"
          aria-label="Filter sort"
        >
          <svg className="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M3 4h18M3 10h12M3 16h6" />
          </svg>
        </button>

        {/* Search Bar */}
        <div className="relative flex flex-1 items-center">
          <svg
            className="pointer-events-none absolute left-2.5 h-3.5 w-3.5 text-zinc-500"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={2}
          >
            <circle cx="11" cy="11" r="8" />
            <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-4.35-4.35" />
          </svg>
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t('demo.searchPlaceholder')}
            className="w-full rounded-md border border-zinc-800 bg-zinc-950 py-1 pl-8 pr-7 font-mono text-caption-12-regular text-zinc-200 placeholder:text-zinc-500 focus:border-zinc-600 focus:outline-none focus:ring-1 focus:ring-zinc-600"
          />
          {query && (
            <button
              type="button"
              onClick={() => setQuery('')}
              className="absolute right-2 text-zinc-400 hover:text-zinc-200 text-xs"
              title="Clear search"
            >
              ✕
            </button>
          )}
        </div>

        {hasKilled && (
          <button
            type="button"
            onClick={resetDemo}
            className="rounded bg-zinc-800 px-2 py-1 text-caption-11-regular font-medium text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100"
            title={t('demo.reset')}
          >
            {t('demo.reset')}
          </button>
        )}

        <div className="flex items-center gap-1">
          <span className="h-2.5 w-2.5 rounded-full bg-zinc-700" title="Menu" />
        </div>
      </div>

      {/* Table Column Headers */}
      <div className="grid grid-cols-[1fr_56px_68px_56px] items-center border-b border-zinc-900 bg-zinc-950/90 px-3.5 py-1.5 font-mono text-[10.5px] font-medium tracking-wider text-zinc-500 uppercase">
        <span>{t('demo.colApp')}</span>
        <span className="text-right">{t('demo.colCpu')}</span>
        <span className="text-right">{t('demo.colRam')}</span>
        <span className="text-right">ACT</span>
      </div>

      {/* Interactive Process List */}
      <div className="max-h-[360px] overflow-y-auto px-1 py-1.5">
        
        {/* SECTION: APPLICATION */}
        <div className="mb-1">
          <div className="px-2.5 py-1 font-mono text-[10px] font-semibold tracking-wider text-zinc-500 uppercase">
            {t('demo.secApp')}
          </div>

          <div className="flex flex-col gap-0.5">
            {liveApps.map((app) => {
              const isExpanded = expandedApps.has(app.id) || Boolean(q)
              return (
                <div key={app.id} className="rounded-md transition-colors hover:bg-zinc-900/60">
                  {/* Parent Row */}
                  <div
                    onClick={() => toggleApp(app.id)}
                    className="grid grid-cols-[1fr_56px_68px_56px] items-center cursor-pointer select-none px-2.5 py-1.5 transition-colors"
                  >
                    <div className="flex items-center gap-2 overflow-hidden pr-2">
                      <span className="text-zinc-500 transition-transform text-[11px]">
                        {isExpanded ? '▾' : '▸'}
                      </span>
                      <AppIcon type={app.iconType} />
                      <span className="truncate text-caption-13-medium font-medium text-zinc-200">
                        {app.name}
                      </span>
                      <span className="rounded bg-zinc-800/80 px-1.5 py-0.2 font-mono text-[10.5px] text-zinc-400">
                        {t('demo.procs', { count: app.aliveCount })}
                      </span>
                    </div>

                    <span className="font-mono text-caption-12-regular text-right text-zinc-300">
                      {app.totalCpu.toFixed(1)}%
                    </span>

                    <span className="font-mono text-caption-12-regular text-right text-zinc-300">
                      {(app.totalMemMb / 1024).toFixed(1)} GB
                    </span>

                    <div className="flex justify-end">
                      <button
                        type="button"
                        onClick={(e) => killApp(app, e)}
                        className="flex h-5 w-5 items-center justify-center rounded text-zinc-500 hover:bg-zinc-800 hover:text-red-400"
                        title={t('demo.terminate')}
                      >
                        ✕
                      </button>
                    </div>
                  </div>

                  {/* Expanded Children Rows */}
                  <AnimatePresence>
                    {isExpanded && (
                      <motion.div
                        initial={{ opacity: 0, height: 0 }}
                        animate={{ opacity: 1, height: 'auto' }}
                        exit={{ opacity: 0, height: 0 }}
                        transition={{ duration: 0.15 }}
                        className="overflow-hidden pl-7 pr-1"
                      >
                        {app.matchingProcs.map((proc) => (
                          <div
                            key={proc.pid}
                            className="grid grid-cols-[1fr_56px_68px_56px] items-center border-l border-zinc-800/80 py-1 pl-3 pr-2 text-zinc-400 transition-colors hover:bg-zinc-900/80"
                          >
                            <div className="flex items-center gap-1.5 overflow-hidden pr-2">
                              <span className="truncate text-caption-12-regular text-zinc-300">
                                {proc.name}
                              </span>
                              <span className="font-mono text-[10px] text-zinc-500">
                                · {proc.pid}
                              </span>
                            </div>

                            <span className="font-mono text-[11px] text-right text-zinc-400">
                              {proc.cpu.toFixed(1)}%
                            </span>

                            <span className="font-mono text-[11px] text-right text-zinc-400">
                              {proc.memMb} MB
                            </span>

                            <div className="flex items-center justify-end gap-1">
                              <button
                                type="button"
                                onClick={(e) => {
                                  e.stopPropagation()
                                  setSelectedProcess(proc)
                                }}
                                className="flex h-4.5 w-4.5 items-center justify-center rounded text-zinc-500 hover:bg-zinc-800 hover:text-zinc-200"
                                title={t('demo.inspect')}
                              >
                                ⓘ
                              </button>
                              <button
                                type="button"
                                onClick={(e) => killProcess(proc.pid, e)}
                                className="flex h-4.5 w-4.5 items-center justify-center rounded text-zinc-500 hover:bg-zinc-800 hover:text-red-400"
                                title={t('demo.terminate')}
                              >
                                ■
                              </button>
                            </div>
                          </div>
                        ))}
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
              )
            })}
          </div>
        </div>

        {/* SECTION: BACKGROUND */}
        <div className="mt-2 border-t border-zinc-900/80 pt-1.5">
          <div
            onClick={() => setExpandBg(!expandBg)}
            className="flex cursor-pointer select-none items-center gap-1.5 px-2.5 py-1 font-mono text-[10px] font-semibold tracking-wider text-zinc-500 uppercase hover:text-zinc-400"
          >
            <span>{expandBg ? '▾' : '▸'}</span>
            <span>{t('demo.secBg')} ({liveBg.length})</span>
          </div>

          <AnimatePresence>
            {expandBg && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
                transition={{ duration: 0.15 }}
                className="overflow-hidden"
              >
                {liveBg.map((proc) => (
                  <div
                    key={proc.pid}
                    className="grid grid-cols-[1fr_56px_68px_56px] items-center px-3 py-1 text-zinc-400 hover:bg-zinc-900/60"
                  >
                    <div className="flex items-center gap-1.5 truncate">
                      <span className="truncate text-caption-12-regular text-zinc-300">
                        {proc.name}
                      </span>
                      <span className="font-mono text-[10px] text-zinc-500">· {proc.pid}</span>
                    </div>
                    <span className="font-mono text-[11px] text-right text-zinc-400">
                      {proc.cpu.toFixed(1)}%
                    </span>
                    <span className="font-mono text-[11px] text-right text-zinc-400">
                      {proc.memMb} MB
                    </span>
                    <div className="flex items-center justify-end gap-1">
                      <button
                        type="button"
                        onClick={() => setSelectedProcess(proc)}
                        className="flex h-4.5 w-4.5 items-center justify-center rounded text-zinc-500 hover:bg-zinc-800 hover:text-zinc-200"
                        title={t('demo.inspect')}
                      >
                        ⓘ
                      </button>
                      <button
                        type="button"
                        onClick={(e) => killProcess(proc.pid, e)}
                        className="flex h-4.5 w-4.5 items-center justify-center rounded text-zinc-500 hover:bg-zinc-800 hover:text-red-400"
                        title={t('demo.terminate')}
                      >
                        ■
                      </button>
                    </div>
                  </div>
                ))}
              </motion.div>
            )}
          </AnimatePresence>
        </div>

        {/* SECTION: SYSTEM */}
        <div className="mt-1 border-t border-zinc-900/80 pt-1.5">
          <div
            onClick={() => setExpandSys(!expandSys)}
            className="flex cursor-pointer select-none items-center gap-1.5 px-2.5 py-1 font-mono text-[10px] font-semibold tracking-wider text-zinc-500 uppercase hover:text-zinc-400"
          >
            <span>{expandSys ? '▾' : '▸'}</span>
            <span>{t('demo.secSys')} ({liveSys.length})</span>
          </div>

          <AnimatePresence>
            {expandSys && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
                transition={{ duration: 0.15 }}
                className="overflow-hidden"
              >
                {liveSys.map((proc) => (
                  <div
                    key={proc.pid}
                    className="grid grid-cols-[1fr_56px_68px_56px] items-center px-3 py-1 text-zinc-400 hover:bg-zinc-900/60"
                  >
                    <div className="flex items-center gap-1.5 truncate">
                      <span className="truncate text-caption-12-regular text-zinc-300">
                        {proc.name}
                      </span>
                      <span className="font-mono text-[10px] text-zinc-500">· {proc.pid}</span>
                    </div>
                    <span className="font-mono text-[11px] text-right text-zinc-400">
                      {proc.cpu.toFixed(1)}%
                    </span>
                    <span className="font-mono text-[11px] text-right text-zinc-400">
                      {proc.memMb} MB
                    </span>
                    <div className="flex items-center justify-end gap-1">
                      <button
                        type="button"
                        onClick={() => setSelectedProcess(proc)}
                        className="flex h-4.5 w-4.5 items-center justify-center rounded text-zinc-500 hover:bg-zinc-800 hover:text-zinc-200"
                        title={t('demo.inspect')}
                      >
                        ⓘ
                      </button>
                      <button
                        type="button"
                        onClick={(e) => killProcess(proc.pid, e)}
                        className="flex h-4.5 w-4.5 items-center justify-center rounded text-zinc-500 hover:bg-zinc-800 hover:text-red-400"
                        title={t('demo.terminate')}
                      >
                        ■
                      </button>
                    </div>
                  </div>
                ))}
              </motion.div>
            )}
          </AnimatePresence>
        </div>

      </div>

      {/* Adwaita Status Bar */}
      <div className="flex items-center justify-between border-t border-zinc-900 bg-zinc-950 px-3.5 py-2 font-mono text-[11px] text-zinc-400">
        <span className="flex items-center gap-1.5 text-zinc-500">
          <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse" />
          <span>cgroup v2 · 1s</span>
        </span>
        <span className="text-zinc-300">
          {t('demo.status', { cpu: totalStats.cpu, ram: totalStats.ram, procs: totalStats.procs })}
        </span>
      </div>

      {/* Inspection Modal (Process Details) */}
      <AnimatePresence>
        {selectedProcess && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 z-20 flex items-center justify-center bg-black/75 p-4 backdrop-blur-xs"
            onClick={() => setSelectedProcess(null)}
          >
            <motion.div
              initial={{ scale: 0.95, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.95, opacity: 0 }}
              onClick={(e) => e.stopPropagation()}
              className="w-full max-w-[360px] rounded-xl border border-zinc-800 bg-zinc-900 p-4 shadow-2xl"
            >
              <div className="flex items-start justify-between border-b border-zinc-800 pb-3">
                <div>
                  <h4 className="text-body-14-medium font-medium text-zinc-100">
                    {selectedProcess.name}
                  </h4>
                  <span className="font-mono text-caption-11-regular text-zinc-400">
                    PID {selectedProcess.pid} · UID {selectedProcess.uid}
                  </span>
                </div>
                <button
                  type="button"
                  onClick={() => setSelectedProcess(null)}
                  className="rounded p-1 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
                  aria-label="Close dialog"
                >
                  ✕
                </button>
              </div>

              <div className="my-3 space-y-2 font-mono text-caption-12-regular text-zinc-300">
                <div className="flex justify-between border-b border-zinc-800/60 pb-1">
                  <span className="text-zinc-500">{t('demo.state')}</span>
                  <span>{selectedProcess.state}</span>
                </div>
                <div className="flex justify-between border-b border-zinc-800/60 pb-1">
                  <span className="text-zinc-500">{t('demo.threads')}</span>
                  <span>{selectedProcess.threads} threads</span>
                </div>
                <div className="flex justify-between border-b border-zinc-800/60 pb-1">
                  <span className="text-zinc-500">Memory RSS</span>
                  <span>{selectedProcess.memMb} MB</span>
                </div>
                <div className="flex justify-between border-b border-zinc-800/60 pb-1">
                  <span className="text-zinc-500">Memory PSS</span>
                  <span>{selectedProcess.pssMb} MB</span>
                </div>
                <div className="pt-1">
                  <span className="text-zinc-500 text-[11px] block">{t('demo.cmdline')}:</span>
                  <p className="mt-1 break-all rounded bg-zinc-950 p-2 text-[10.5px] text-zinc-300 select-all">
                    {selectedProcess.cmdline}
                  </p>
                </div>
              </div>

              <div className="mt-4 flex gap-2">
                <button
                  type="button"
                  onClick={() => killProcess(selectedProcess.pid)}
                  className="flex-1 rounded-md bg-red-600/20 py-1.5 font-mono text-caption-12-medium text-red-300 transition-colors hover:bg-red-600/30"
                >
                  Terminate (SIGTERM)
                </button>
                <button
                  type="button"
                  onClick={() => setSelectedProcess(null)}
                  className="rounded-md border border-zinc-800 px-3 py-1.5 text-caption-12-medium text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
                >
                  {t('demo.close')}
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Interactive Helper Ribbon */}
      <div className="bg-zinc-900/90 px-3 py-1.5 text-center font-mono text-[10.5px] text-zinc-400 border-t border-zinc-800/60">
        💡 {t('demo.hint')}
      </div>
    </div>
  )
}
