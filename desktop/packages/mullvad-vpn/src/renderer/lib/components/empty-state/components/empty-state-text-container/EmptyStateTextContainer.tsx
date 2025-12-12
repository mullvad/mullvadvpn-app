import { Flex, FlexProps } from '../../../flex';

export type EmptyStateTextContainerProps = FlexProps;

export function EmptyStateTextContainer({ children, ...props }: EmptyStateTextContainerProps) {
  return (
    <Flex flexDirection="column" alignItems="center" gap="tiny" {...props}>
      {children}
    </Flex>
  );
}
