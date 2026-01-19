import { useCallback } from 'react';

import { messages } from '../../../../../../shared/gettext';
import { FilterAccordion } from '../../../../FilterAccordion';
import { CheckboxRow } from '../checkbox-row';

interface IFilterByProviderProps {
  providers: Record<string, boolean>;
  availableOptions: string[];
  setProviders: (providers: (previous: Record<string, boolean>) => Record<string, boolean>) => void;
}

export function FilterByProvider(props: IFilterByProviderProps) {
  const { setProviders } = props;

  const onToggle = useCallback(
    (provider: string) =>
      setProviders((providers) => {
        const newProviders = { ...providers, [provider]: !providers[provider] };
        return props.availableOptions.every((provider) => newProviders[provider])
          ? toggleAllProviders(providers, true)
          : newProviders;
      }),
    [props.availableOptions, setProviders],
  );

  const toggleAll = useCallback(() => {
    setProviders((providers) => toggleAllProviders(providers));
  }, [setProviders]);

  return (
    <FilterAccordion title={messages.pgettext('filter-view', 'Providers')}>
      <CheckboxRow
        label={messages.pgettext('filter-view', 'All providers')}
        $bold
        checked={Object.values(props.providers).every((value) => value)}
        onChange={toggleAll}
      />
      {Object.entries(props.providers)
        .filter(([provider]) => props.availableOptions.includes(provider))
        .map(([provider, checked]) => (
          <CheckboxRow key={provider} label={provider} checked={checked} onChange={onToggle} />
        ))}
    </FilterAccordion>
  );
}

function toggleAllProviders(providers: Record<string, boolean>, value?: boolean) {
  const shouldSelect = value ?? !Object.values(providers).every((value) => value);
  return Object.fromEntries(Object.keys(providers).map((provider) => [provider, shouldSelect]));
}
