import { Text, TextProps } from '../../Text';

export type FootnoteMiniSemiBoldProps<E extends React.ElementType = 'span'> = TextProps<E>;

export function FootnoteMiniSemiBold<T extends React.ElementType = 'span'>(
  props: FootnoteMiniSemiBoldProps<T>,
) {
  return <Text variant="footnoteMiniSemiBold" {...props} />;
}
