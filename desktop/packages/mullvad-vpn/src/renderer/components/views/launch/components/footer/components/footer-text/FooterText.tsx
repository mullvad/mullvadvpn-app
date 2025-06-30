import { Text, TextProps } from '../../../../../../../lib/components';

export type FooterTextProps = TextProps;

export function FooterText(props: FooterTextProps) {
  return <Text variant="labelTiny" {...props}></Text>;
}
