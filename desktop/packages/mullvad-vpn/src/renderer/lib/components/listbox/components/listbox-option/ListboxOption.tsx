import { ListItem, ListItemProps } from '../../../list-item';
import { ListItemTriggerProps } from '../../../list-item/components';
import { ListboxOptionLabel } from './components';
import { ListboxOptionProvider } from './components/listbox-option-context/ListboxOptionContext';
import { ListboxOptionIcon } from './components/listbox-option-icon';
import { ListboxOptionItem } from './components/listbox-option-item/ListboxOptionItem';
import { ListboxOptionTrigger } from './components/listbox-option-trigger/ListboxOptionTrigger';

export type ListboxOptionProps<T> = ListItemProps &
  Pick<ListItemTriggerProps, 'onClick'> & {
    value: T;
  };

function ListboxOption<T>({ value, children, ...props }: ListboxOptionProps<T>) {
  return (
    <ListboxOptionProvider value={value}>
      <ListItem {...props}>{children}</ListItem>
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
