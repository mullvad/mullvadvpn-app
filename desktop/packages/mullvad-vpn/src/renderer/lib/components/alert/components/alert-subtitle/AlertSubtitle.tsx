import { Text, TextProps } from '../../../text';

export type AlertSubtitleProps = TextProps;

export function AlertSubtitle({ children, ...props }: AlertSubtitleProps) {
  return (
    <Text variant="bodySmallSemibold" color="white" {...props}>
      {children}
    </Text>
  );
}
