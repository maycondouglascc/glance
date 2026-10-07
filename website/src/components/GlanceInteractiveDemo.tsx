import { useState, useMemo } from 'react'
import { motion, AnimatePresence } from 'motion/react'

interface SubProcess {
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

interface AppGroup {
  id: string
  name: string
  iconSrc: string
  processes: SubProcess[]
}

interface FlatProcess {
  pid: number
  name: string
  iconSrc?: string
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
    iconSrc: './icons/firefox.svg',
    processes: [
      {
        pid: 19220,
        name: 'Firefox',
        cmdline: '/usr/lib/firefox/firefox --new-window',
        exe: '/usr/lib/firefox/firefox',
        cpu: 1.2,
        memMb: 340,
        pssMb: 290,
        threads: 64,
        uid: 1000,
        state: 'Running (R)',
      },
      {
        pid: 19221,
        name: 'Firefox',
        cmdline: '/usr/lib/firefox/firefox -contentproc -childID 1',
        exe: '/usr/lib/firefox/firefox',
        cpu: 1.8,
        memMb: 420,
        pssMb: 380,
        threads: 32,
        uid: 1000,
        state: 'Sleeping (S)',
      },
      {
        pid: 19222,
        name: 'Firefox',
        cmdline: '/usr/lib/firefox/firefox -contentproc -childID 2',
        exe: '/usr/lib/firefox/firefox',
        cpu: 1.2,
        memMb: 380,
        pssMb: 340,
        threads: 28,
        uid: 1000,
        state: 'Sleeping (S)',
      },
    ],
  },
  {
    id: 'vscode',
    name: 'Visual Studio Code',
    iconSrc: './icons/vscode.svg',
    processes: [
      {
        pid: 24102,
        name: 'code',
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
    iconSrc: './icons/spotify.svg',
    processes: [
      {
        pid: 14201,
        name: 'spotify',
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
    id: 'alacritty',
    name: 'Alacritty',
    iconSrc: './icons/alacritty.svg',
    processes: [
      {
        pid: 8812,
        name: 'alacritty',
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
    iconSrc: './icons/docker.svg',
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
    name: '1password',
    iconSrc: './icons/1password.svg',
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
    iconSrc: './icons/tailscale.svg',
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
    iconSrc: './icons/linux.svg',
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
    name: 'gnome-shell',
    iconSrc: './icons/gnome.svg',
    cmdline: '/usr/bin/gnome-shell',
    exe: '/usr/bin/gnome-shell',
    cpu: 1.1,
    memMb: 210,
    pssMb: 180,
    threads: 16,
    uid: 1000,
    state: 'Running (R)',
  },
]

export function GlanceInteractiveDemo() {
  const [query, setQuery] = useState('')
  const [expandedApps, setExpandedApps] = useState<Set<string>>(new Set(['firefox']))
  const [expandBg, setExpandBg] = useState(false)
  const [expandSys, setExpandSys] = useState(false)
  const [killedPids, setKilledPids] = useState<Set<number>>(new Set())
  const [selectedProcess, setSelectedProcess] = useState<SubProcess | FlatProcess | null>(null)
  const [sortBy, setSortBy] = useState<'cpu' | 'memory' | 'count' | 'name'>('cpu')
  const [showSortMenu, setShowSortMenu] = useState(false)
  const [showMainMenu, setShowMainMenu] = useState(false)
  const [showAboutModal, setShowAboutModal] = useState(false)
  const [isMinimized, setIsMinimized] = useState(false)

  const toggleApp = (id: string) => {
    setExpandedApps((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  const killProcess = (pid: number, e?: React.MouseEvent) => {
    e?.stopPropagation()
    setKilledPids((prev) => new Set(prev).add(pid))
    if (selectedProcess?.pid === pid) {
      setSelectedProcess(null)
    }
  }

  const killApp = (app: AppGroup, e: React.MouseEvent) => {
    e.stopPropagation()
    setKilledPids((prev) => {
      const next = new Set(prev)
      app.processes.forEach((p) => next.add(p.pid))
      return next
    })
  }

  const resetDemo = () => {
    setKilledPids(new Set())
    setExpandedApps(new Set(['firefox']))
    setQuery('')
    setSelectedProcess(null)
  }

  const q = query.trim().toLowerCase()

  const liveApps = useMemo(() => {
    const list = INITIAL_APPS.map((app) => {
      const aliveProcs = app.processes.filter((p) => !killedPids.has(p.pid))
      const matchingProcs = q
        ? aliveProcs.filter(
            (p) =>
              p.name.toLowerCase().includes(q) ||
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

    list.sort((a, b) => {
      if (sortBy === 'cpu') return b.totalCpu - a.totalCpu
      if (sortBy === 'memory') return b.totalMemMb - a.totalMemMb
      if (sortBy === 'count') return b.matchingProcs.length - a.matchingProcs.length
      if (sortBy === 'name') return a.name.localeCompare(b.name)
      return 0
    })

    return list
  }, [killedPids, q, sortBy])

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
      cpu: (cpu + 7.5).toFixed(1), // system base cpu
      ram: ((memMb + 1400) / 1024).toFixed(1),
      procs: count + 140, // realistic system totals
    }
  }, [liveApps, liveBg, liveSys])

  const hasKilled = killedPids.size > 0

  return (
    <div className="relative mx-auto w-full max-w-[442px] select-none overflow-hidden rounded-[24px] bg-[#18181b] font-ibm-sans text-[#f0f0f0] shadow-2xl ring-1 ring-white/5">
      
      {/* 1:1 GTK Headerbar (32px height, flat transparent) */}
      <div className="flex items-center justify-between px-2.5 pt-2.5 pb-1.5">
        
        {/* Sort Menu Button (32x32 circular #202026) */}
        <div className="relative">
          <button
            type="button"
            onClick={() => {
              setShowSortMenu((prev) => !prev)
              setShowMainMenu(false)
            }}
            className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[#202026] text-[#a1a1aa] transition-colors hover:bg-[#2c2c34] hover:text-[#f0f0f0]"
            title="Sort order"
            aria-label="Sort order"
          >
            {/* Exact GTK sort icon */}
            <svg className="h-4 w-4" viewBox="0 0 16 16" fill="currentColor">
              <path d="M2 3.5h7v1H2zm0 4h5v1H2zm0 4h3v1H2zm10.5-8.5l2.5 3h-2v9h-1v-9h-2z" />
            </svg>
          </button>

          {showSortMenu && (
            <div className="absolute left-0 top-10 z-30 w-48 rounded-xl border border-[#2c2c34] bg-[#202026] p-1.5 shadow-2xl text-[12px] font-ibm-sans">
              <button
                type="button"
                onClick={() => { setSortBy('cpu'); setShowSortMenu(false); }}
                className={`flex w-full items-center justify-between rounded-lg px-2.5 py-1.5 text-left transition-colors ${sortBy === 'cpu' ? 'bg-[#2c2c34] text-white font-medium' : 'text-[#a1a1aa] hover:bg-white/[0.04] hover:text-[#f0f0f0]'}`}
              >
                <span>Sort by CPU</span>
                {sortBy === 'cpu' && <span className="text-xs">✓</span>}
              </button>
              <button
                type="button"
                onClick={() => { setSortBy('memory'); setShowSortMenu(false); }}
                className={`flex w-full items-center justify-between rounded-lg px-2.5 py-1.5 text-left transition-colors ${sortBy === 'memory' ? 'bg-[#2c2c34] text-white font-medium' : 'text-[#a1a1aa] hover:bg-white/[0.04] hover:text-[#f0f0f0]'}`}
              >
                <span>Sort by Memory</span>
                {sortBy === 'memory' && <span className="text-xs">✓</span>}
              </button>
              <button
                type="button"
                onClick={() => { setSortBy('count'); setShowSortMenu(false); }}
                className={`flex w-full items-center justify-between rounded-lg px-2.5 py-1.5 text-left transition-colors ${sortBy === 'count' ? 'bg-[#2c2c34] text-white font-medium' : 'text-[#a1a1aa] hover:bg-white/[0.04] hover:text-[#f0f0f0]'}`}
              >
                <span>Sort by Process Count</span>
                {sortBy === 'count' && <span className="text-xs">✓</span>}
              </button>
              <button
                type="button"
                onClick={() => { setSortBy('name'); setShowSortMenu(false); }}
                className={`flex w-full items-center justify-between rounded-lg px-2.5 py-1.5 text-left transition-colors ${sortBy === 'name' ? 'bg-[#2c2c34] text-white font-medium' : 'text-[#a1a1aa] hover:bg-white/[0.04] hover:text-[#f0f0f0]'}`}
              >
                <span>Sort by Name</span>
                {sortBy === 'name' && <span className="text-xs">✓</span>}
              </button>
            </div>
          )}
        </div>

        {/* Center Search Entry (260x32, #202026, 12px) */}
        <div className="relative mx-2 flex h-8 flex-1 max-w-[260px] items-center">
          <svg
            className="pointer-events-none absolute left-2.5 h-4 w-4 text-[#a1a1aa]"
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
            placeholder="Search apps or PID…"
            className="h-8 w-full rounded-full border-none bg-[#202026] pl-8 pr-7 text-[12px] text-[#f0f0f0] placeholder-[#8e8e8e] outline-none focus:ring-1 focus:ring-white/20"
          />
          {query && (
            <button
              type="button"
              onClick={() => setQuery('')}
              className="absolute right-2.5 text-xs text-[#a1a1aa] hover:text-[#f0f0f0]"
            >
              ✕
            </button>
          )}
        </div>

        {/* Right actions: 3-dots Menu + Close Window Button */}
        <div className="flex items-center gap-1.5">
          {hasKilled && (
            <button
              type="button"
              onClick={resetDemo}
              className="h-6 rounded-full bg-[#202026] px-2 text-[10px] font-medium text-[#a1a1aa] hover:bg-[#2c2c34] hover:text-[#f0f0f0]"
              title="Reset processes"
            >
              Reset
            </button>
          )}
          
          {/* Main 3-dots Menu */}
          <div className="relative">
            <button
              type="button"
              onClick={() => {
                setShowMainMenu((prev) => !prev)
                setShowSortMenu(false)
              }}
              className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[#202026] text-[#a1a1aa] transition-colors hover:bg-[#2c2c34] hover:text-[#f0f0f0]"
              title="Main menu"
              aria-label="Main menu"
            >
              <svg className="h-4 w-4" viewBox="0 0 16 16" fill="currentColor">
                <circle cx="8" cy="3.5" r="1.5" />
                <circle cx="8" cy="8" r="1.5" />
                <circle cx="8" cy="12.5" r="1.5" />
              </svg>
            </button>

            {showMainMenu && (
              <div className="absolute right-0 top-10 z-30 w-44 rounded-xl border border-[#2c2c34] bg-[#202026] p-1.5 shadow-2xl text-[12px] font-ibm-sans">
                <button
                  type="button"
                  onClick={() => {
                    setShowAboutModal(true)
                    setShowMainMenu(false)
                  }}
                  className="flex w-full items-center rounded-lg px-2.5 py-1.5 text-left text-[#a1a1aa] transition-colors hover:bg-white/[0.04] hover:text-[#f0f0f0]"
                >
                  About Glance
                </button>
                <button
                  type="button"
                  onClick={() => {
                    resetDemo()
                    setShowMainMenu(false)
                  }}
                  className="flex w-full items-center rounded-lg px-2.5 py-1.5 text-left text-[#a1a1aa] transition-colors hover:bg-white/[0.04] hover:text-[#f0f0f0]"
                >
                  Reset Processes
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setIsMinimized(true)
                    setShowMainMenu(false)
                  }}
                  className="flex w-full items-center rounded-lg px-2.5 py-1.5 text-left text-[#a1a1aa] transition-colors hover:bg-white/[0.04] hover:text-[#f0f0f0]"
                >
                  Minimize to Tray
                </button>
              </div>
            )}
          </div>

          {/* Close / Minimize Button */}
          <button
            type="button"
            onClick={() => setIsMinimized(true)}
            className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[#202026] text-[#a1a1aa] transition-colors hover:bg-[#2c2c34] hover:text-[#f0f0f0]"
            title="Close Glance"
            aria-label="Close Glance"
          >
            <svg className="h-3.5 w-3.5" viewBox="0 0 16 16" fill="currentColor">
              <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z" />
            </svg>
          </button>
        </div>
      </div>

      {/* 1:1 Sticky Column Header */}
      <div className="flex items-center px-4 pt-1 pb-1.5 font-ibm-mono text-[10px] font-bold tracking-[1px] uppercase">
        <span className="text-[#a1a1aa]">APPLICATION</span>
        <div className="ml-auto flex items-center">
          <span className="w-[36px] text-right text-[#787878]">CPU</span>
          <span className="ml-[24px] w-[38px] text-right text-[#787878]">RAM</span>
          <div className="w-[44px] ml-[16px]" />
        </div>
      </div>

      {/* Process Scrolled Container */}
      <div className="max-h-[380px] overflow-y-auto px-3 pb-2 pt-0.5 scrollbar-thin">
        
        {/* APPLICATION Group */}
        <div className="flex flex-col gap-0.5">
          {liveApps.map((app) => {
            const isExpanded = expandedApps.has(app.id) || Boolean(q)
            return (
              <div key={app.id} className="app-group-card">
                
                {/* Application Header Row */}
                <div
                  onClick={() => toggleApp(app.id)}
                  className="flex h-7 cursor-pointer items-center rounded-md px-1 transition-colors hover:bg-white/[0.04]"
                >
                  {/* Chevron button */}
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation()
                      toggleApp(app.id)
                    }}
                    className="flex h-5 w-5 shrink-0 items-center justify-center text-[#a1a1aa] hover:text-[#f0f0f0]"
                  >
                    <span className="text-[10px]">{isExpanded ? '▾' : '▸'}</span>
                  </button>

                  {/* Official SVG Logo */}
                  <img
                    src={app.iconSrc}
                    alt={app.name}
                    className="h-5 w-5 shrink-0 object-contain ml-1"
                    loading="lazy"
                  />

                  {/* App Title */}
                  <span className="ml-1.5 truncate text-[12px] font-semibold text-[#f0f0f0]">
                    {app.name}
                  </span>

                  {/* Process count pill */}
                  <span className="ml-2 rounded-[2px] bg-[#202026] px-2 py-0.5 text-[12px] font-medium tracking-[0.5px] text-[#9e9e9e]">
                    {app.aliveCount} procs
                  </span>

                  {/* Right metrics */}
                  <div className="ml-auto flex items-center">
                    <span className="w-[36px] text-right font-ibm-mono text-[12px] font-semibold text-[#f0f0f0]">
                      {app.totalCpu.toFixed(1)}%
                    </span>
                    <span className="ml-[24px] w-[38px] text-right font-ibm-mono text-[12px] font-semibold text-[#f0f0f0]">
                      {(app.totalMemMb / 1024).toFixed(1)} GB
                    </span>

                    {/* App Terminate Button */}
                    <div className="ml-[16px] flex w-[44px] justify-end">
                      <button
                        type="button"
                        onClick={(e) => killApp(app, e)}
                        className="flex h-5 w-5 items-center justify-center rounded-full bg-[#202026] text-[#a1a1aa] transition-colors hover:bg-[#2c2c34] hover:text-[#f0f0f0]"
                        title="End application"
                      >
                        <svg className="h-3 w-3" viewBox="0 0 16 16" fill="currentColor">
                          <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z" />
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>

                {/* Progressive Disclosure Children */}
                <AnimatePresence>
                  {isExpanded && (
                    <motion.div
                      initial={{ opacity: 0, height: 0 }}
                      animate={{ opacity: 1, height: 'auto' }}
                      exit={{ opacity: 0, height: 0 }}
                      transition={{ duration: 0.12 }}
                      className="ml-[14px] mb-1 border-l-[1.5px] border-[#202026] pl-[10px]"
                    >
                      {app.matchingProcs.map((proc) => (
                        <div
                          key={proc.pid}
                          className="flex h-[26px] items-center rounded-md px-1 transition-colors hover:bg-white/[0.04]"
                        >
                          <span className="truncate text-[12px] font-medium text-[#a1a1aa]">
                            {proc.name}  ·  {proc.pid}
                          </span>

                          <div className="ml-auto flex items-center">
                            <span className="w-[36px] text-right font-ibm-mono text-[12px] font-medium text-[#a1a1aa]">
                              {proc.cpu.toFixed(1)}%
                            </span>
                            <span className="ml-[24px] w-[38px] text-right font-ibm-mono text-[12px] font-medium text-[#a1a1aa]">
                              {(proc.memMb >= 1024 ? `${(proc.memMb / 1024).toFixed(1)} GB` : `${proc.memMb} MB`)}
                            </span>

                            {/* Process Action Buttons: Info & Terminate */}
                            <div className="ml-[16px] flex w-[44px] items-center justify-end gap-1">
                              <button
                                type="button"
                                onClick={() => setSelectedProcess(proc)}
                                className="flex h-5 w-5 items-center justify-center rounded-full bg-[#202026] text-[#a1a1aa] transition-colors hover:bg-[#2c2c34] hover:text-[#f0f0f0]"
                                title="Inspect process details"
                              >
                                <svg className="h-3 w-3" viewBox="0 0 16 16" fill="currentColor">
                                  <circle cx="8" cy="8" r="7" fill="none" stroke="currentColor" strokeWidth="1.5" />
                                  <circle cx="8" cy="5" r="1" />
                                  <path d="M7 7h1v4h1" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
                                </svg>
                              </button>
                              <button
                                type="button"
                                onClick={(e) => killProcess(proc.pid, e)}
                                className="flex h-5 w-5 items-center justify-center rounded-full bg-[#202026] text-[#a1a1aa] transition-colors hover:bg-[#2c2c34] hover:text-[#f0f0f0]"
                                title="End process (SIGTERM)"
                              >
                                <svg className="h-2.5 w-2.5" viewBox="0 0 16 16" fill="currentColor">
                                  <rect x="3" y="3" width="10" height="10" rx="1.5" />
                                </svg>
                              </button>
                            </div>
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

        {/* BACKGROUND Collapsible Section */}
        <div className="mt-2.5">
          <div
            onClick={() => setExpandBg(!expandBg)}
            className="flex cursor-pointer items-center gap-1.5 px-1 py-1 font-ibm-mono text-[10px] font-bold tracking-[1px] text-[#a1a1aa] uppercase hover:text-[#f0f0f0]"
          >
            <span>{expandBg ? '▾' : '▸'}</span>
            <span>BACKGROUND</span>
          </div>

          <AnimatePresence>
            {expandBg && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
                transition={{ duration: 0.12 }}
                className="ml-[14px] mb-1 border-l-[1.5px] border-[#202026] pl-[10px]"
              >
                {liveBg.map((proc) => (
                  <div
                    key={proc.pid}
                    className="flex h-[26px] items-center rounded-md px-1 transition-colors hover:bg-white/[0.04]"
                  >
                    <div className="flex items-center gap-1.5 truncate">
                      {proc.iconSrc && (
                        <img src={proc.iconSrc} alt={proc.name} className="h-4 w-4 shrink-0 object-contain" />
                      )}
                      <span className="truncate text-[12px] font-medium text-[#a1a1aa]">
                        {proc.name}  ·  {proc.pid}
                      </span>
                    </div>

                    <div className="ml-auto flex items-center">
                      <span className="w-[36px] text-right font-ibm-mono text-[12px] font-medium text-[#a1a1aa]">
                        {proc.cpu.toFixed(1)}%
                      </span>
                      <span className="ml-[24px] w-[38px] text-right font-ibm-mono text-[12px] font-medium text-[#a1a1aa]">
                        {proc.memMb} MB
                      </span>
                      <div className="ml-[16px] flex w-[44px] items-center justify-end gap-1">
                        <button
                          type="button"
                          onClick={() => setSelectedProcess(proc)}
                          className="flex h-5 w-5 items-center justify-center rounded-full bg-[#202026] text-[#a1a1aa] hover:bg-[#2c2c34] hover:text-[#f0f0f0]"
                        >
                          <svg className="h-3 w-3" viewBox="0 0 16 16" fill="currentColor">
                            <circle cx="8" cy="8" r="7" fill="none" stroke="currentColor" strokeWidth="1.5" />
                            <circle cx="8" cy="5" r="1" />
                            <path d="M7 7h1v4h1" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
                          </svg>
                        </button>
                        <button
                          type="button"
                          onClick={(e) => killProcess(proc.pid, e)}
                          className="flex h-5 w-5 items-center justify-center rounded-full bg-[#202026] text-[#a1a1aa] hover:bg-[#2c2c34] hover:text-[#f0f0f0]"
                        >
                          <svg className="h-2.5 w-2.5" viewBox="0 0 16 16" fill="currentColor">
                            <rect x="3" y="3" width="10" height="10" rx="1.5" />
                          </svg>
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </motion.div>
            )}
          </AnimatePresence>
        </div>

        {/* SYSTEM Collapsible Section */}
        <div className="mt-1">
          <div
            onClick={() => setExpandSys(!expandSys)}
            className="flex cursor-pointer items-center gap-1.5 px-1 py-1 font-ibm-mono text-[10px] font-bold tracking-[1px] text-[#a1a1aa] uppercase hover:text-[#f0f0f0]"
          >
            <span>{expandSys ? '▾' : '▸'}</span>
            <span>SYSTEM</span>
          </div>

          <AnimatePresence>
            {expandSys && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
                transition={{ duration: 0.12 }}
                className="ml-[14px] mb-1 border-l-[1.5px] border-[#202026] pl-[10px]"
              >
                {liveSys.map((proc) => (
                  <div
                    key={proc.pid}
                    className="flex h-[26px] items-center rounded-md px-1 transition-colors hover:bg-white/[0.04]"
                  >
                    <div className="flex items-center gap-1.5 truncate">
                      {proc.iconSrc && (
                        <img src={proc.iconSrc} alt={proc.name} className="h-4 w-4 shrink-0 object-contain" />
                      )}
                      <span className="truncate text-[12px] font-medium text-[#a1a1aa]">
                        {proc.name}  ·  {proc.pid}
                      </span>
                    </div>

                    <div className="ml-auto flex items-center">
                      <span className="w-[36px] text-right font-ibm-mono text-[12px] font-medium text-[#a1a1aa]">
                        {proc.cpu.toFixed(1)}%
                      </span>
                      <span className="ml-[24px] w-[38px] text-right font-ibm-mono text-[12px] font-medium text-[#a1a1aa]">
                        {proc.memMb} MB
                      </span>
                      <div className="ml-[16px] flex w-[44px] items-center justify-end gap-1">
                        <button
                          type="button"
                          onClick={() => setSelectedProcess(proc)}
                          className="flex h-5 w-5 items-center justify-center rounded-full bg-[#202026] text-[#a1a1aa] hover:bg-[#2c2c34] hover:text-[#f0f0f0]"
                        >
                          <svg className="h-3 w-3" viewBox="0 0 16 16" fill="currentColor">
                            <circle cx="8" cy="8" r="7" fill="none" stroke="currentColor" strokeWidth="1.5" />
                            <circle cx="8" cy="5" r="1" />
                            <path d="M7 7h1v4h1" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
                          </svg>
                        </button>
                        <button
                          type="button"
                          onClick={(e) => killProcess(proc.pid, e)}
                          className="flex h-5 w-5 items-center justify-center rounded-full bg-[#202026] text-[#a1a1aa] hover:bg-[#2c2c34] hover:text-[#f0f0f0]"
                        >
                          <svg className="h-2.5 w-2.5" viewBox="0 0 16 16" fill="currentColor">
                            <rect x="3" y="3" width="10" height="10" rx="1.5" />
                          </svg>
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </motion.div>
            )}
          </AnimatePresence>
        </div>

      </div>

      {/* 1:1 GTK Footer Status Bar (border-top 1px solid #202026, text-[#a1a1aa] 12px 500) */}
      <div className="border-t border-[#202026] px-4 py-3.5 text-right font-ibm-mono text-[12px] font-medium text-[#a1a1aa]">
        CPU {totalStats.cpu}%  ·  RAM {totalStats.ram} GB  ·  {totalStats.procs} processes
      </div>

      {/* Process Details Popover (matching Glance adw::Window inspection) */}
      <AnimatePresence>
        {selectedProcess && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 z-30 flex items-center justify-center bg-black/80 p-4 backdrop-blur-xs"
            onClick={() => setSelectedProcess(null)}
          >
            <motion.div
              initial={{ scale: 0.95, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.95, opacity: 0 }}
              onClick={(e) => e.stopPropagation()}
              className="w-full max-w-[340px] rounded-[16px] border border-[#2c2c34] bg-[#18181b] p-4 text-[#f0f0f0] shadow-2xl"
            >
              <div className="flex items-center justify-between border-b border-[#202026] pb-2.5">
                <span className="text-[13px] font-semibold text-[#f0f0f0]">
                  {selectedProcess.name}
                </span>
                <button
                  type="button"
                  onClick={() => setSelectedProcess(null)}
                  className="h-6 w-6 rounded-full text-[#a1a1aa] hover:bg-[#202026] hover:text-[#f0f0f0]"
                >
                  ✕
                </button>
              </div>

              <div className="my-3 space-y-1.5 font-ibm-mono text-[11px] text-[#a1a1aa]">
                <div className="flex justify-between py-0.5 border-b border-[#202026]">
                  <span className="text-[#787878]">PID</span>
                  <span className="text-[#f0f0f0]">{selectedProcess.pid}</span>
                </div>
                <div className="flex justify-between py-0.5 border-b border-[#202026]">
                  <span className="text-[#787878]">State</span>
                  <span className="text-[#f0f0f0]">{selectedProcess.state}</span>
                </div>
                <div className="flex justify-between py-0.5 border-b border-[#202026]">
                  <span className="text-[#787878]">Threads</span>
                  <span className="text-[#f0f0f0]">{selectedProcess.threads}</span>
                </div>
                <div className="flex justify-between py-0.5 border-b border-[#202026]">
                  <span className="text-[#787878]">Memory RSS</span>
                  <span className="text-[#f0f0f0]">{selectedProcess.memMb} MB</span>
                </div>
                <div className="flex justify-between py-0.5 border-b border-[#202026]">
                  <span className="text-[#787878]">Memory PSS</span>
                  <span className="text-[#f0f0f0]">{selectedProcess.pssMb} MB</span>
                </div>
                <div className="pt-1">
                  <span className="text-[#787878] block">Command:</span>
                  <p className="mt-1 break-all rounded bg-[#202026] p-2 text-[10px] text-[#f0f0f0]">
                    {selectedProcess.cmdline}
                  </p>
                </div>
              </div>

              <div className="mt-3 flex gap-2">
                <button
                  type="button"
                  onClick={() => killProcess(selectedProcess.pid)}
                  className="flex-1 rounded-[6px] bg-red-600/20 py-1.5 font-ibm-mono text-[11px] font-semibold text-red-400 hover:bg-red-600/30"
                >
                  Terminate (SIGTERM)
                </button>
                <button
                  type="button"
                  onClick={() => setSelectedProcess(null)}
                  className="rounded-[6px] bg-[#202026] px-3 py-1.5 text-[11px] text-[#a1a1aa] hover:bg-[#2c2c34] hover:text-[#f0f0f0]"
                >
                  Close
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Minimized Tray View (When window is closed) */}
      <AnimatePresence>
        {isMinimized && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 z-40 flex flex-col items-center justify-center bg-[#18181b]/95 p-6 backdrop-blur-xs text-center"
          >
            <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-[#202026] text-sm font-semibold text-white shadow-inner">
              G
            </div>
            <span className="mt-3 text-[13px] font-medium text-[#f0f0f0]">
              Glance running in system tray
            </span>
            <span className="mt-1 font-ibm-mono text-[11px] text-[#a1a1aa]">
              0.00% CPU · 65 MB RAM
            </span>
            <button
              type="button"
              onClick={() => setIsMinimized(false)}
              className="mt-4 rounded-full bg-[#202026] px-4 py-1.5 text-[12px] font-medium text-[#f0f0f0] transition-colors hover:bg-[#2c2c34]"
            >
              Restore Window
            </button>
          </motion.div>
        )}
      </AnimatePresence>

      {/* About Glance Modal */}
      <AnimatePresence>
        {showAboutModal && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 z-40 flex items-center justify-center bg-black/80 p-4 backdrop-blur-xs"
            onClick={() => setShowAboutModal(false)}
          >
            <motion.div
              initial={{ scale: 0.95, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.95, opacity: 0 }}
              onClick={(e) => e.stopPropagation()}
              className="w-full max-w-[300px] rounded-[18px] border border-[#2c2c34] bg-[#18181b] p-5 text-center shadow-2xl font-ibm-sans"
            >
              <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-2xl bg-zinc-800 text-lg font-bold text-white shadow-inner">
                G
              </div>
              <h3 className="mt-3 text-[15px] font-semibold text-[#f0f0f0]">Glance</h3>
              <span className="text-[11px] font-ibm-mono text-[#a1a1aa]">v0.1.0</span>
              <p className="mt-2 text-[12px] text-[#a1a1aa] leading-relaxed">
                Grouped process monitor for Linux. Progressive disclosure and true zero idle CPU.
              </p>
              <div className="mt-3 pt-3 border-t border-[#202026] text-[11px] text-[#787878]">
                Created by Maycon Douglas
              </div>
              <button
                type="button"
                onClick={() => setShowAboutModal(false)}
                className="mt-4 w-full rounded-lg bg-[#202026] py-1.5 text-[12px] font-medium text-[#f0f0f0] hover:bg-[#2c2c34]"
              >
                Close
              </button>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

    </div>
  )
}
