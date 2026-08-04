import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';
import { useFilteredProviders } from '../../../filter/hooks';
import { useActiveOwnership, useActiveProviders } from '../../hooks';
import { useScrollPositionContext } from '../../ScrollPositionContext';

export type ProvidersFilterChip = FilterChipProps;

export function ProvidersFilterChip(props: ProvidersFilterChip) {
  const { resetScrollPositions } = useScrollPositionContext();
  const relaySettings = useNormalRelaySettings();
  const { activeProviders, setActiveProviders } = useActiveProviders();
  const { activeOwnership } = useActiveOwnership();
  const filteredProviders = useFilteredProviders(activeProviders, activeOwnership);

  const onClearProviders = React.useCallback(async () => {
    resetScrollPositions();
    if (relaySettings) {
      await setActiveProviders([]);
    }
  }, [resetScrollPositions, relaySettings, setActiveProviders]);

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
