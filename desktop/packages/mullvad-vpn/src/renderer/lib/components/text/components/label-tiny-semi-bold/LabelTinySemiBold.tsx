import { Text, TextProps } from '../../Text';

export type LabelTinySemiBoldProps<E extends React.ElementType = 'span'> = TextProps<E>;

export function LabelTinySemiBold<T extends React.ElementType = 'span'>(
  props: LabelTinySemiBoldProps<T>,
) {
  return <Text variant="labelTinySemiBold" {...props} />;
}
