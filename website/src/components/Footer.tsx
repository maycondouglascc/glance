import { useLanguage } from '../context/LanguageContext'

export default function Footer() {
  const { t } = useLanguage()

  return (
    <footer className="mt-12 border-t border-zinc-200 pt-6 dark:border-zinc-800">
      <h2 className="text-body-15-medium font-medium text-zinc-900 dark:text-zinc-100">
        {t('footer.contact')}
      </h2>
      <ul className="mt-4 flex flex-wrap items-center gap-4 text-body-15-medium font-medium">
        <li>
          <a
            href="https://maycondouglas.work"
            target="_blank"
            rel="noopener noreferrer"
            className="text-accent dark:text-accent font-medium hover:underline"
          >
            {t('footer.portfolio')}
          </a>
        </li>
        <li>
          <a
            href="https://github.com/maycondouglascc/glance"
            target="_blank"
            rel="noopener noreferrer"
            className="text-zinc-900 dark:text-zinc-100 font-medium hover:underline"
          >
            {t('footer.github')}
          </a>
        </li>
        <li>
          <a
            href="https://linkedin.com/in/maycondouglascc"
            target="_blank"
            rel="noopener noreferrer"
            className="text-zinc-900 dark:text-zinc-100 font-medium hover:underline"
          >
            {t('footer.linkedin')}
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
