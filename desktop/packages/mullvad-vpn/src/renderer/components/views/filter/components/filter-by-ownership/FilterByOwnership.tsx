import { useMemo } from 'react';

import { Ownership } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { FilterAccordion } from '../../../../FilterAccordion';
import { SettingsListbox } from '../../../../settings-listbox';

interface IFilterByOwnershipProps {
  ownership: Ownership;
  availableOptions: Ownership[];
  setOwnership: (ownership: Ownership) => void;
}

export function FilterByOwnership({
  availableOptions,
  ownership,
  setOwnership,
}: IFilterByOwnershipProps) {
  const values = useMemo(
    () =>
      [
        {
          label: messages.pgettext('filter-view', 'Mullvad owned only'),
          value: Ownership.mullvadOwned,
        },
        {
          label: messages.pgettext('filter-view', 'Rented only'),
          value: Ownership.rented,
        },
      ].filter((option) => availableOptions.includes(option.value)),
    [availableOptions],
  );

  return (
    <FilterAccordion title={messages.pgettext('filter-view', 'Ownership')}>
      <SettingsListbox value={ownership} onValueChange={setOwnership}>
        <SettingsListbox.Options>
          <SettingsListbox.BaseOption value={Ownership.any}>
            {messages.gettext('Any')}
          </SettingsListbox.BaseOption>
          {values.map((option) => (
            <SettingsListbox.BaseOption key={option.value} value={option.value}>
              {option.label}
            </SettingsListbox.BaseOption>
          ))}
        </SettingsListbox.Options>
      </SettingsListbox>
    </FilterAccordion>
  );
}
