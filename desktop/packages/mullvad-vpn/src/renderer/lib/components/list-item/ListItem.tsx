import { Flex } from '../flex';
import {
  ListItemContent,
  ListItemFooter,
  ListItemGroup,
  ListItemItem,
  ListItemLabel,
  ListItemText,
  ListItemTrigger,
} from './components';
import { levels } from './levels';
import { ListItemProvider } from './ListItemContext';

export interface ListItemProps {
  level?: keyof typeof levels;
  disabled?: boolean;
  children: React.ReactNode;
}

const ListItem = ({ level = 0, disabled, children }: ListItemProps) => {
  return (
    <ListItemProvider level={level} disabled={disabled}>
      <Flex $flexDirection="column" $gap="tiny">
        {children}
      </Flex>
    </ListItemProvider>
  );
};

const ListItemNamespace = Object.assign(ListItem, {
  Content: ListItemContent,
  Label: ListItemLabel,
  Group: ListItemGroup,
  Text: ListItemText,
  Trigger: ListItemTrigger,
  Item: ListItemItem,
  Footer: ListItemFooter,
});

export { ListItemNamespace as ListItem };
