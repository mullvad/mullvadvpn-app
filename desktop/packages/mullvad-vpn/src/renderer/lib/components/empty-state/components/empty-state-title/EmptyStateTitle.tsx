import { Text, TextProps } from '../../../text';

export type EmptyStateTitleProps = TextProps;

export function EmptyStateTitle({ children, ...props }: EmptyStateTitleProps) {
  return (
    <Text variant="titleMedium" textAlign="center" {...props}>
      {children}
    </Text>
  );
}
