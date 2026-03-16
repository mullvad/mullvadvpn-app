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
  Footer: Listbox.Footer,
  Options: Listbox.Options,
  BaseOption: BaseOption,
  InputOption: InputOption,
  SplitOption: SplitOption,
  CheckboxOption: CheckboxOption,
});

export { SettingsListboxNamespace as SettingsListbox };
