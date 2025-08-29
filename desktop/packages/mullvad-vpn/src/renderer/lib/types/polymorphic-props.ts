export type PolymorphicProps<E extends React.ElementType, Props = object> = Props &
  Omit<React.ComponentPropsWithoutRef<E>, keyof Props> & {
    as?: E;
  };
