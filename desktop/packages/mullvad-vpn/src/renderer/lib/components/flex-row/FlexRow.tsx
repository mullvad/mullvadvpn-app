import { Flex, FlexProps } from '../flex';

type FlexRowProps<T extends React.ElementType = 'div'> = FlexProps<T>;

export const FlexRow = <T extends React.ElementType = 'div'>(props: FlexRowProps<T>) => (
  <Flex flexDirection="row" {...props} />
);
