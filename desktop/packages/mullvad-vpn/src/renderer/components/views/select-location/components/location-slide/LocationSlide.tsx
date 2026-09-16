import { type HTMLMotionProps, motion } from 'motion/react';

export type LocationSlideProps = HTMLMotionProps<'div'>;

export function LocationSlide({ children, ...props }: LocationSlideProps) {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.25 }}
      {...props}>
      {children}
    </motion.div>
  );
}
