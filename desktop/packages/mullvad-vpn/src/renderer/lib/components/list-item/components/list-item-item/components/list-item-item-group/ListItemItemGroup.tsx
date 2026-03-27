import { Flex, FlexProps } from '../../../../../flex';

export type ListItemItemGroupProps = FlexProps;

export const ListItemItemGroup = (props: ListItemItemGroupProps) => {
  return <Flex alignItems="center" {...props} />;
};
