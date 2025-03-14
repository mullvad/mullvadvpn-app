import { Text, TextProps } from './Text';

export type FootnoteMiniProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const FootnoteMini = <T extends React.ElementType = 'span'>(props: FootnoteMiniProps<T>) => (
  <Text variant="footnoteMini" {...props} />
);
