import React from 'react';

import { ScrollToAnchorId } from '../../../shared/ipc-types';
import { useScrollToListItem } from '../../hooks';
import { Listbox, ListboxProps } from '../../lib/components/listbox';
import { BaseOption, InputOption, SplitOption } from './components';

export type SettingsListboxProps<T> = Omit<ListboxProps<T>, 'animation'> & {
  anchorId?: ScrollToAnchorId;
};

function SettingsListbox<T>({ anchorId, children, ...props }: SettingsListboxProps<T>) {
  const { ref, animation } = useScrollToListItem(anchorId);
  const labelId = React.useId();

  return (
    <Listbox
      ref={ref}
      tabIndex={-1}
      role="region"
      labelId={labelId}
      aria-labelledby={labelId}
      animation={animation}
      {...props}>
      {children}
    </Listbox>
  );
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
  InputOption: InputOption,
  SplitOption: SplitOption,
});

export { SettingsListboxNamespace as SettingsListbox };
