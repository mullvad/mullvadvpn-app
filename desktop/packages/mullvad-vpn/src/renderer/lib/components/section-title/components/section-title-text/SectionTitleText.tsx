import { Text, TextProps } from '../../../text';

export type SectionTitleTextProps = TextProps;

export function SectionTitleText(props: SectionTitleTextProps) {
  return <Text variant="labelTiny" color="whiteAlpha60" {...props} />;
}
