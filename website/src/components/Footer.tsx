import { useLanguage } from '../context/LanguageContext'

export default function Footer() {
  const { t } = useLanguage()

  return (
    <footer className="mt-12 pt-6">
      <ul className="flex flex-wrap items-center gap-4 text-body-15-medium font-medium">
        <li>
          <a
            href="https://maycondouglas.work"
            target="_blank"
            rel="noopener noreferrer"
            className="text-accent dark:text-accent font-medium hover:underline"
          >
            {t('footer.madeWith')}
          </a>
        </li>

        <li>
          <a
            href="https://github.com/maycondouglascc/glance/blob/main/LICENSE"
            target="_blank"
            rel="noopener noreferrer"
            className="text-zinc-600 dark:text-zinc-400 font-normal hover:underline"
          >
            {t('footer.license')}
          </a>
        </li>
      </ul>
    </footer>
  )
}
