import { Text, TextProps } from './Text';

export type LabelTinySemiBoldProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const LabelTinySemiBold = <T extends React.ElementType = 'span'>(
  props: LabelTinySemiBoldProps<T>,
) => <Text variant="labelTinySemiBold" {...props} />;
