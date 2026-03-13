import React from 'react';
import { sprintf } from 'sprintf-js';

import { Ownership } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';
import { useFilteredProviders } from '../../../filter/hooks';
import { useScrollPositionContext } from '../../ScrollPositionContext';

export type ProvidersFilterChip = FilterChipProps;

export function ProvidersFilterChip(props: ProvidersFilterChip) {
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const { resetScrollPositions } = useScrollPositionContext();
  const relaySettings = useNormalRelaySettings();
  const ownership = relaySettings?.ownership ?? Ownership.any;
  const providers = relaySettings?.providers ?? [];
  const filteredProviders = useFilteredProviders(providers, ownership);

  const onClearProviders = React.useCallback(async () => {
    resetScrollPositions();
    if (relaySettings) {
      await relaySettingsUpdater((settings) => ({ ...settings, providers: [] }));
    }
  }, [relaySettingsUpdater, resetScrollPositions, relaySettings]);

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
