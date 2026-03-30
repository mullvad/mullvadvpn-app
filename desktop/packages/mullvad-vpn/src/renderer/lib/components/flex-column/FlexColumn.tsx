import { Flex, FlexProps } from '../flex';

export type FlexColumnProps<T extends React.ElementType = 'div'> = FlexProps<T>;

export const FlexColumn = <T extends React.ElementType = 'div'>(props: FlexColumnProps<T>) => (
  <Flex flexDirection="column" {...props} />
);
