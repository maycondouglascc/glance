import { motion } from 'motion/react'
import { ReactNode } from 'react'

interface TransitionChildProps {
  children: ReactNode
  className?: string
  index?: number
}

export function TransitionChild({ children, className, index = 0 }: TransitionChildProps) {
  return (
    <motion.div
      className={className}
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{
        delay: 0.05 + index * 0.07,
        duration: 0.35,
        ease: 'easeOut',
      }}
    >
      {children}
    </motion.div>
  )
}
