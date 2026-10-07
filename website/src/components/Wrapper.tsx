import { ReactNode } from 'react'

interface WrapperProps {
  children: ReactNode
}

export default function Wrapper({ children }: WrapperProps) {
  return (
    <div className="palette-transition relative mx-auto w-full animate-fade-in rounded-lg bg-white p-6 sm:p-10 shadow-sm motion-reduce:animate-none dark:bg-zinc-900 dark:shadow-none dark:ring-1 dark:ring-zinc-800">
      {children}
    </div>
  )
}
