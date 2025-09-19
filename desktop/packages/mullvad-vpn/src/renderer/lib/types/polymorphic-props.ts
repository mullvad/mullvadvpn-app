export type PolymorphicProps<E extends React.ElementType, Props = object> = Props &
  Omit<React.ComponentPropsWithRef<E>, keyof Props> & {
    as?: E;
  };
