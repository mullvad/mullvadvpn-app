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
          <SettingsListbox.Options.CheckboxOption
            value={availableProviders}
            checked={allProvidersSelected}>
            {messages.pgettext('filter-view', 'All providers')}
          </SettingsListbox.Options.CheckboxOption>
          {availableProviders.map((provider) => (
            <SettingsListbox.Options.CheckboxOption
              key={provider}
              value={provider}
              checked={selectedProviders?.includes(provider)}>
              {provider}
            </SettingsListbox.Options.CheckboxOption>
          ))}
        </SettingsListbox.Options>
      </SettingsListbox>
    </FilterAccordion>
  );
}
