import { Flex, FlexProps } from '../../flex';

export type ListItemFooterProps = FlexProps;

export const ListItemFooter = (props: ListItemFooterProps) => {
  return <Flex $padding={{ horizontal: 'medium' }} {...props} />;
};
