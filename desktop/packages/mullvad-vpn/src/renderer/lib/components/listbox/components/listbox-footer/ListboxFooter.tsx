import { Flex, FlexProps } from '../../../flex';
import { ListboxFooterText } from './components';

export type ListboxFooterProps = FlexProps;

const ListboxFooter = (props: ListboxFooterProps) => {
  return <Flex padding={{ horizontal: 'medium' }} margin={{ top: 'tiny' }} {...props} />;
};

const ListboxFooterNamespace = Object.assign(ListboxFooter, {
  Text: ListboxFooterText,
});

export { ListboxFooterNamespace as ListboxFooter };
