import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { useSelectedEntryOrExitLocation } from '../../../../../features/locations/hooks';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';
import { useFilteredProviders } from '../../../filter/hooks';
import { useActiveOwnership, useActiveProviders } from '../../hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

export type ProvidersFilterChipProps = FilterChipProps;

export function ProvidersFilterChip(props: ProvidersFilterChipProps) {
  const relaySettings = useNormalRelaySettings();
  const { activeProviders, setActiveProviders } = useActiveProviders();
  const { activeOwnership } = useActiveOwnership();
  const filteredProviders = useFilteredProviders(activeProviders, activeOwnership);

  const { locationType } = useSelectLocationViewContext();
  const location = useSelectedEntryOrExitLocation(locationType);
  const inactive = location === 'any'; // Filter is not applied when `any` location is used

  const label = inactive
    ? // This line is here to prevent the following one to be moved up here by prettier
      // TRANSLATORS: Accessibility description for button removing the providers filter.
      messages.pgettext('accessibility', 'Remove providers filter')
    : // This line is here to prevent the following one to be moved up here by prettier
      // TRANSLATORS: Accessibility description for button to remove the providers filter
      // TRANSLATORS: when the filter is currently not active.
      messages.pgettext('accessibility', 'Remove providers filter (inactive)');

  const onClearProviders = React.useCallback(async () => {
    if (relaySettings) {
      await setActiveProviders([]);
    }
  }, [relaySettings, setActiveProviders]);

  return (
    <FilterChip aria-label={label} onClick={onClearProviders} inactive={inactive} {...props}>
      <FilterChip.Text>
        {sprintf(messages.pgettext('select-location-view', 'Providers: %(numberOfProviders)d'), {
          numberOfProviders: filteredProviders.length,
        })}
      </FilterChip.Text>
      <FilterChip.Icon icon="cross" />
    </FilterChip>
  );
}
