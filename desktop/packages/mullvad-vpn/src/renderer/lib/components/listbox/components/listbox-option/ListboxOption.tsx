import { ListItem, ListItemProps } from '../../../list-item';
import {
  ListboxOptionIcon,
  ListboxOptionItem,
  ListboxOptionLabel,
  ListboxOptionTrigger,
} from './components';
import { ListboxOptionProvider } from './ListboxOptionContext';

export type ListboxOptionProps<T> = ListItemProps & {
  value: T;
};

function ListboxOption<T>({ value, children, ...props }: ListboxOptionProps<T>) {
  return (
    <ListboxOptionProvider value={value}>
      <ListItem level={1} {...props}>
        {children}
      </ListItem>
    </ListboxOptionProvider>
  );
}

const ListboxOptionNamespace = Object.assign(ListboxOption, {
  Content: ListItem.Content,
  Group: ListItem.Group,
  Trigger: ListboxOptionTrigger,
  Item: ListboxOptionItem,
  Footer: ListItem.Footer,
  Icon: ListboxOptionIcon,
  Label: ListboxOptionLabel,
});

export { ListboxOptionNamespace as ListboxOption };
