import { Text, TextProps } from './Text';

export type FoonoteMiniProps<E extends React.ElementType> = TextProps<E>;

export const FootnoteMini = <T extends React.ElementType = 'span'>(props: FoonoteMiniProps<T>) => (
  <Text variant="footnoteMini" {...props} />
);
