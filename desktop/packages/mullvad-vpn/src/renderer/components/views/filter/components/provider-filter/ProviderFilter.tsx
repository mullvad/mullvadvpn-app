import { messages } from '../../../../../../shared/gettext';
import { FilterAccordion, FilterAccordionProps } from '../../../../FilterAccordion';
import { SettingsListbox } from '../../../../settings-listbox';
import { useFilterViewContext } from '../../FilterViewContext';

export type ProviderFilterProps = FilterAccordionProps;

export function ProviderFilter(props: ProviderFilterProps) {
  const { availableProviders, selectedProviders, toggleProviders } = useFilterViewContext();

  const allProvidersSelected = availableProviders.every((provider) =>
    selectedProviders?.includes(provider),
  );

  return (
    <FilterAccordion title={messages.pgettext('filter-view', 'Providers')} {...props}>
      <SettingsListbox value={selectedProviders} onValueChange={toggleProviders}>
        <SettingsListbox.Options>
          <SettingsListbox.CheckboxOption value={availableProviders} checked={allProvidersSelected}>
            {messages.pgettext('filter-view', 'All providers')}
          </SettingsListbox.CheckboxOption>
          {availableProviders.map((provider) => (
            <SettingsListbox.CheckboxOption
              key={provider}
              value={provider}
              checked={selectedProviders?.includes(provider)}>
              {provider}
            </SettingsListbox.CheckboxOption>
          ))}
        </SettingsListbox.Options>
      </SettingsListbox>
    </FilterAccordion>
  );
}
