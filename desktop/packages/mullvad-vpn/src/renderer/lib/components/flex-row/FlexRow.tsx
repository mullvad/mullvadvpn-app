import { Flex, FlexProps } from '../flex';

export type FlexRowProps<T extends React.ElementType = 'div'> = FlexProps<T>;

export const FlexRow = <T extends React.ElementType = 'div'>(props: FlexRowProps<T>) => (
  <Flex flexDirection="row" {...props} />
);
