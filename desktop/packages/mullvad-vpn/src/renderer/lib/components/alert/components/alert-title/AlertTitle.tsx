import { Text, TextProps } from '../../../text';

export type AlertTitleProps = TextProps;

export function AlertTitle({ children, ...props }: AlertTitleProps) {
  return (
    <Text variant="titleLarge" color="white" {...props}>
      {children}
    </Text>
  );
}
