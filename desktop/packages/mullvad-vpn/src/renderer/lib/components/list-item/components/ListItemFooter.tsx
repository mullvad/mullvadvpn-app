import { Flex, FlexProps } from '../../flex';

export type ListIOtemFooterProps = FlexProps;

export const ListItemFooter = (props: ListIOtemFooterProps) => {
  return <Flex $padding={{ horizontal: 'medium' }} {...props} />;
};
