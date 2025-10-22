import { Flex, FlexProps } from '../../../flex';

export type DialogTextGroupProps = FlexProps;

export function AlertTextGroup({ children, ...props }: FlexProps) {
  return (
    <Flex $flexDirection="column" $gap="small" {...props}>
      {children}
    </Flex>
  );
}
