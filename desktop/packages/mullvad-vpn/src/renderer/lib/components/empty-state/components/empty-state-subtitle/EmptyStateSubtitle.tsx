import { Text, TextProps } from '../../../text';

export type EmptyStateSubtitleProps = TextProps;
export function EmptyStateSubtitle({ children, ...props }: EmptyStateSubtitleProps) {
  return (
    <Text variant="labelTiny" color="whiteAlpha60" textAlign="center" {...props}>
      {children}
    </Text>
  );
}
