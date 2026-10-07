import LanguageSelector from './LanguageSelector'
import { ThemeToggle } from './ThemeToggle'

export default function SettingsBar() {
  return (
    <div
      className="inline-flex items-center gap-1.5 rounded-md bg-zinc-100 px-2 py-1 dark:bg-zinc-900 border border-zinc-200/60 dark:border-zinc-800/80 shadow-xs"
      role="group"
      aria-label="Settings"
    >
      <LanguageSelector />
      <span className="h-3.5 w-px bg-zinc-300 dark:bg-zinc-700" aria-hidden="true" />
      <ThemeToggle />
    </div>
  )
}
