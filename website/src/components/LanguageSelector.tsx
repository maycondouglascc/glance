import { useLanguage, type Language } from '../context/LanguageContext'

const options: { value: Language; shortLabel: string; longLabel: string }[] = [
  { value: 'en', shortLabel: 'EN', longLabel: 'English' },
  { value: 'pt', shortLabel: 'PT', longLabel: 'Português' },
]

export default function LanguageSelector() {
  const { language, setLanguage } = useLanguage()

  return (
    <div
      role="radiogroup"
      aria-label="Language selector"
      className="inline-flex items-center gap-0.5"
    >
      {options.map(({ value, shortLabel, longLabel }) => {
        const isActive = language === value
        return (
          <button
            key={value}
            type="button"
            role="radio"
            aria-checked={isActive}
            aria-label={longLabel}
            title={longLabel}
            onClick={() => setLanguage(value)}
            className={`h-7 w-7 rounded-md text-caption-11-regular font-medium transition-colors duration-200 ${
              isActive
                ? 'bg-white text-zinc-900 shadow-xs dark:bg-zinc-800 dark:text-zinc-100'
                : 'text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100'
            }`}
          >
            {shortLabel}
          </button>
        )
      })}
    </div>
  )
}
