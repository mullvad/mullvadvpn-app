import { ScrollToAnchorId } from '../../../shared/ipc-types';
import { useScrollToListItem } from '../../hooks';
import { Listbox, ListboxProps } from '../../lib/components/listbox/Listbox';
import { BaseOption, InputListboxOption, SplitListboxOption } from './components';

export type SettingsListboxProps<T> = Omit<ListboxProps<T>, 'animation'> & {
  anchorId?: ScrollToAnchorId;
};

function SettingsListbox<T>({ anchorId, ...props }: SettingsListboxProps<T>) {
  const { ref, animation } = useScrollToListItem(anchorId);

  return <Listbox ref={ref} animation={animation} {...props} />;
}

const SettingsListboxNamespace = Object.assign(SettingsListbox, {
  Item: Listbox.Item,
  Content: Listbox.Content,
  Label: Listbox.Label,
  Group: Listbox.Group,
  Text: Listbox.Text,
  Footer: Listbox.Footer,
  Icon: Listbox.Icon,
  Option: Listbox.Option,
  Options: Listbox.Options,
  BaseOption: BaseOption,
  InputOption: InputListboxOption,
  SplitOption: SplitListboxOption,
});

export { SettingsListboxNamespace as SettingsListbox };
