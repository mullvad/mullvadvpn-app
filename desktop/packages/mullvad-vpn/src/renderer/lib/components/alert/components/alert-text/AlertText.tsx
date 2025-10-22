import { Text, TextProps } from '../../../text';

export type AlertTextProps = TextProps;

export function AlertText({ children, ...props }: AlertTextProps) {
  return (
    <Text variant="labelTinySemiBold" color="whiteAlpha60" {...props}>
      {children}
    </Text>
  );
}
