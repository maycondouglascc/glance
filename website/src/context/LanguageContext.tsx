import { createContext, useCallback, useContext, useMemo, useState } from 'react'
import { en } from '../locales/en'
import { pt } from '../locales/pt'

export type Language = 'en' | 'pt'

type DeepStringRecord<T> = {
  [K in keyof T]: T[K] extends object ? DeepStringRecord<T[K]> : string
}

type LocaleShape = DeepStringRecord<typeof en>
type TranslationKey = DotPath<typeof en>

type DotPath<T> = T extends object
  ? {
      [K in keyof T & string]: T[K] extends object ? `${K}` | `${K}.${DotPath<T[K]>}` : `${K}`
    }[keyof T & string]
  : never

const STORAGE_KEY = 'glance-language'
const translations: Record<Language, LocaleShape> = { en, pt }

type LanguageContextValue = {
  language: Language
  setLanguage: (value: Language) => void
  t: (key: TranslationKey, params?: Record<string, string | number>) => string
}

const LanguageContext = createContext<LanguageContextValue | undefined>(undefined)

function getStoredOrDefaultLanguage(): Language {
  if (typeof window === 'undefined') return 'en'
  const stored = window.localStorage.getItem(STORAGE_KEY)
  if (stored === 'en' || stored === 'pt') return stored
  return window.navigator.language.toLowerCase().startsWith('pt') ? 'pt' : 'en'
}

function getByPath(obj: unknown, path: string): unknown {
  if (!obj || typeof obj !== 'object') return undefined
  return path.split('.').reduce<unknown>((acc, segment) => {
    if (!acc || typeof acc !== 'object') return undefined
    return (acc as Record<string, unknown>)[segment]
  }, obj)
}

function interpolate(template: string, params?: Record<string, string | number>) {
  if (!params) return template
  return template.replace(/\{(\w+)\}/g, (_, key: string) => String(params[key] ?? `{${key}}`))
}

export function LanguageProvider({ children }: { children: React.ReactNode }) {
  const [language, setLanguageState] = useState<Language>(getStoredOrDefaultLanguage)

  const setLanguage = useCallback((value: Language) => {
    setLanguageState(value)
    window.localStorage.setItem(STORAGE_KEY, value)
  }, [])

  const value = useMemo<LanguageContextValue>(() => {
    return {
      language,
      setLanguage,
      t: (key: TranslationKey, params?: Record<string, string | number>) => {
        const activeDict = translations[language]
        const fallbackDict = translations.en

        const activeValue = getByPath(activeDict, key)
        if (typeof activeValue === 'string') {
          return interpolate(activeValue, params)
        }

        const fallbackValue = getByPath(fallbackDict, key)
        if (typeof fallbackValue === 'string') {
          return interpolate(fallbackValue, params)
        }

        return key
      },
    }
  }, [language, setLanguage])

  return <LanguageContext.Provider value={value}>{children}</LanguageContext.Provider>
}

export function useLanguage() {
  const context = useContext(LanguageContext)
  if (!context) {
    throw new Error('useLanguage must be used within a LanguageProvider')
  }
  return context
}
