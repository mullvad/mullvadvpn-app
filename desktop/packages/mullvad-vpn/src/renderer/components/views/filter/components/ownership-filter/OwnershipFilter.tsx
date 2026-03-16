import { Ownership } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { FilterAccordion, FilterAccordionProps } from '../../../../FilterAccordion';
import { SettingsListbox } from '../../../../settings-listbox';
import { useFilterViewContext } from '../../FilterViewContext';

type OwnershipFilterProps = FilterAccordionProps;

export function OwnershipFilter(props: OwnershipFilterProps) {
  const { selectedOwnership, setOwnership } = useFilterViewContext();
  const values = [
    {
      label: messages.pgettext('filter-view', 'Mullvad owned only'),
      value: Ownership.mullvadOwned,
    },
    {
      label: messages.pgettext('filter-view', 'Rented only'),
      value: Ownership.rented,
    },
  ];

  return (
    <FilterAccordion title={messages.pgettext('filter-view', 'Ownership')} {...props}>
      <SettingsListbox value={selectedOwnership} onValueChange={setOwnership}>
        <SettingsListbox.Options>
          <SettingsListbox.Options.BaseOption value={Ownership.any}>
            {messages.gettext('Any')}
          </SettingsListbox.Options.BaseOption>
          {values.map((option) => (
            <SettingsListbox.Options.BaseOption key={option.value} value={option.value}>
              {option.label}
            </SettingsListbox.Options.BaseOption>
          ))}
        </SettingsListbox.Options>
      </SettingsListbox>
    </FilterAccordion>
  );
}
