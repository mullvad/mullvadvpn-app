import { Text, TextProps } from '../../../text';

export type SectionTitleTitleProps = TextProps;

export function SectionTitleTitle(props: SectionTitleTitleProps) {
  return <Text variant="labelTinySemiBold" {...props} />;
}
