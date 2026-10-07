import { createContext, useContext, useEffect, useState, useCallback } from 'react'

type Theme = 'light' | 'dark' | 'system'

interface ThemeContextValue {
  theme: Theme
  resolvedTheme: 'light' | 'dark'
  setTheme: (theme: Theme) => void
}

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined)

const STORAGE_KEY = 'glance-theme'
const DARK_BG = '#09090b'
const LIGHT_BG = '#ffffff'

function getSystemPreference(): 'light' | 'dark' {
  if (typeof window === 'undefined') return 'dark'
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function resolveTheme(theme: Theme): 'light' | 'dark' {
  if (theme === 'system') return getSystemPreference()
  return theme
}

function applyTheme(resolved: 'light' | 'dark') {
  const root = document.documentElement
  if (resolved === 'dark') {
    root.classList.add('dark')
  } else {
    root.classList.remove('dark')
  }
  root.style.colorScheme = resolved

  const meta = document.querySelector('meta[name="theme-color"]')
  if (meta) {
    meta.setAttribute('content', resolved === 'dark' ? DARK_BG : LIGHT_BG)
  }
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(() => {
    if (typeof window === 'undefined') return 'system'
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored === 'light' || stored === 'dark' || stored === 'system') return stored as Theme
    return 'system'
  })

  const [resolvedTheme, setResolvedTheme] = useState<'light' | 'dark'>(() => resolveTheme(theme))

  const setTheme = useCallback((newTheme: Theme) => {
    setThemeState(newTheme)
    localStorage.setItem(STORAGE_KEY, newTheme)
  }, [])

  useEffect(() => {
    const resolved = resolveTheme(theme)
    setResolvedTheme(resolved)
    applyTheme(resolved)
  }, [theme])

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    function handleChange() {
      if (theme === 'system') {
        const resolved = getSystemPreference()
        setResolvedTheme(resolved)
        applyTheme(resolved)
      }
    }
    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [theme])

  return (
    <ThemeContext.Provider value={{ theme, resolvedTheme, setTheme }}>
      {children}
    </ThemeContext.Provider>
  )
}

export function useTheme() {
  const context = useContext(ThemeContext)
  if (!context) {
    throw new Error('useTheme must be used within a ThemeProvider')
  }
  return context
}
