import { useTheme } from '../context/ThemeContext'

export function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme()

  const toggle = () => {
    setTheme(resolvedTheme === 'dark' ? 'light' : 'dark')
  }

  return (
    <button
      type="button"
      onClick={toggle}
      className="inline-flex h-7 w-7 items-center justify-center rounded-md text-zinc-600 transition-colors hover:text-zinc-900 focus-visible:outline-none dark:text-zinc-400 dark:hover:text-zinc-100"
      aria-label={resolvedTheme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
      title={resolvedTheme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
    >
      {resolvedTheme === 'dark' ? (
        <svg className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
          <circle cx="12" cy="12" r="5" />
          <path strokeLinecap="round" strokeLinejoin="round" d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
        </svg>
      ) : (
        <svg className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z" />
        </svg>
      )}
    </button>
  )
}
