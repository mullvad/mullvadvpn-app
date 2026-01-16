import { Flex, FlexProps } from '../../../flex';

export type ListboxFooterProps = FlexProps;

export const ListboxFooter = (props: ListboxFooterProps) => {
  return <Flex padding={{ horizontal: 'medium' }} margin={{ top: 'tiny' }} {...props} />;
};
