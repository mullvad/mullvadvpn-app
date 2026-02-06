import React from 'react';

import { ScrollToAnchorId } from '../../../shared/ipc-types';
import { Listbox, ListboxProps } from '../../lib/components/listbox';
import {
  BaseOption,
  CheckboxOption,
  InputOption,
  SettingsListboxHeader,
  SplitOption,
} from './components';
import { SettingsListboxProvider } from './SettingsListboxContext';

export type SettingsListboxProps<T> = Omit<ListboxProps<T>, 'animation'> & {
  anchorId?: ScrollToAnchorId;
};

function SettingsListbox<T>({ anchorId, children, ...props }: SettingsListboxProps<T>) {
  const labelId = React.useId();

  return (
    <SettingsListboxProvider anchorId={anchorId}>
      <Listbox labelId={labelId} {...props}>
        {children}
      </Listbox>
    </SettingsListboxProvider>
  );
}

const SettingsListboxNamespace = Object.assign(SettingsListbox, {
  Header: SettingsListboxHeader,
  HeaderItem: Listbox.HeaderItem,
  Label: Listbox.Label,
  Group: Listbox.Group,
  ActionGroup: Listbox.ActionGroup,
  Text: Listbox.Text,
  Footer: Listbox.Footer,
  FooterText: Listbox.FooterText,
  Icon: Listbox.Icon,
  Option: Listbox.Option,
  Options: Listbox.Options,
  BaseOption: BaseOption,
  InputOption: InputOption,
  SplitOption: SplitOption,
  CheckboxOption: CheckboxOption,
});

export { SettingsListboxNamespace as SettingsListbox };
