import { Listbox } from '../../../../lib/components/listbox';
import type { ListboxOptionsProps } from '../../../../lib/components/listbox/components';
import { BaseOption, CheckboxOption, InputOption, SplitOption } from './components';

export type SettingsListboxOptionsProps = ListboxOptionsProps;

function SettingsListboxOptions({ children, ...props }: SettingsListboxOptionsProps) {
  return <Listbox.Options {...props}>{children}</Listbox.Options>;
}

const SettingsListboxOptionsNamespace = Object.assign(SettingsListboxOptions, {
  BaseOption: BaseOption,
  InputOption: InputOption,
  SplitOption: SplitOption,
  CheckboxOption: CheckboxOption,
});

export { SettingsListboxOptionsNamespace as SettingsListboxOptions };
