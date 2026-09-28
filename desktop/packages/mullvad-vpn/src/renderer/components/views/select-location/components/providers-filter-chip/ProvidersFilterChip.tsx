import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';
import { useFilteredProviders } from '../../../filter/hooks';
import { useActiveOwnership, useActiveProviders } from '../../hooks';

export type ProvidersFilterChipProps = FilterChipProps;

export function ProvidersFilterChip(props: ProvidersFilterChipProps) {
  const relaySettings = useNormalRelaySettings();
  const { activeProviders, setActiveProviders } = useActiveProviders();
  const { activeOwnership } = useActiveOwnership();
  const filteredProviders = useFilteredProviders(activeProviders, activeOwnership);

  const onClearProviders = React.useCallback(async () => {
    if (relaySettings) {
      await setActiveProviders([]);
    }
  }, [relaySettings, setActiveProviders]);

  return (
    <FilterChip
      aria-label={
        // TRANSLATORS: Accessibility description for button removing the providers filter.
        messages.pgettext('accessibility', 'Remove providers filter')
      }
      onClick={onClearProviders}
      {...props}>
      <FilterChip.Text>
        {sprintf(messages.pgettext('select-location-view', 'Providers: %(numberOfProviders)d'), {
          numberOfProviders: filteredProviders.length,
        })}
      </FilterChip.Text>
      <FilterChip.Icon icon="cross" />
    </FilterChip>
  );
}
