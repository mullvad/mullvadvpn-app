import { Text, TextProps } from '../../Text';

export type FootnoteMiniProps<E extends React.ElementType = 'span'> = TextProps<E>;

export function FootnoteMini<T extends React.ElementType = 'span'>(props: FootnoteMiniProps<T>) {
  return <Text variant="footnoteMini" {...props} />;
}
