import { Flex, FlexProps } from '../../../flex';

export type DialogButtonGroupProps = FlexProps;

export function DialogButtonGroup({ children, ...props }: DialogButtonGroupProps) {
  return (
    <Flex $flexDirection="column" $gap="small" {...props}>
      {children}
    </Flex>
  );
}
