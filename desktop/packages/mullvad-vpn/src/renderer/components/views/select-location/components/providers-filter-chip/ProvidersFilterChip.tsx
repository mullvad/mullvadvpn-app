import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { useOwnership, useProviders } from '../../../../../features/locations/hooks';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';
import { useFilteredProviders } from '../../../filter/hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

export type ProvidersFilterChip = FilterChipProps;

export function ProvidersFilterChip(props: ProvidersFilterChip) {
  const relaySettings = useNormalRelaySettings();
  const { locationType } = useSelectLocationViewContext();
  const { ownership } = useOwnership(locationType);
  const { activeProviders, providers, setProviders } = useProviders(locationType);
  const filteredProviders = useFilteredProviders(activeProviders, ownership);

  const onClearProviders = React.useCallback(async () => {
    if (relaySettings) {
      await setProviders(providers);
    }
  }, [relaySettings, setProviders, providers]);

  return (
    <FilterChip aria-label={messages.gettext('Clear')} onClick={onClearProviders} {...props}>
      <FilterChip.Text>
        {sprintf(messages.pgettext('select-location-view', 'Providers: %(numberOfProviders)d'), {
          numberOfProviders: filteredProviders.length,
        })}
      </FilterChip.Text>
      <FilterChip.Icon icon="cross" />
    </FilterChip>
  );
}
