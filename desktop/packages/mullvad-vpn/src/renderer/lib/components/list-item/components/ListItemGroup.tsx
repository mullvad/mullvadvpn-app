import { Flex, FlexProps } from '../../flex';

export type ListItemGroupProps = FlexProps;

export const ListItemGroup = (props: ListItemGroupProps) => {
  return <Flex $alignItems="center" $gap="small" {...props} />;
};
