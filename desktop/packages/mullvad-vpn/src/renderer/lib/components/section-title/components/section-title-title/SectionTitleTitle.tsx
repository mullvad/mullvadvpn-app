import { Text, TextProps } from '../../../text';

export type SectionTitleTitleProps<T extends React.ElementType = 'span'> = TextProps<T>;

export function SectionTitleTitle<T extends React.ElementType = 'span'>(
  props: SectionTitleTitleProps<T>,
) {
  return <Text variant="labelTinySemiBold" {...props} />;
}
