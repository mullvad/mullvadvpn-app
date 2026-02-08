import React from 'react';

import { Ownership } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';
import { useScrollPositionContext } from '../../ScrollPositionContext';
import { useOwnershipFilterLabel } from './hooks';

export type OwnershipFilterChipProps = FilterChipProps;

export function OwnershipFilterChip(props: OwnershipFilterChipProps) {
  const relaySettings = useNormalRelaySettings();
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const { resetScrollPositions } = useScrollPositionContext();

  const ownership = relaySettings?.ownership ?? Ownership.any;
  const ownershipFilterLabel = useOwnershipFilterLabel(ownership);

  const onClearOwnership = React.useCallback(async () => {
    resetScrollPositions();
    if (relaySettings) {
      await relaySettingsUpdater((settings) => ({ ...settings, ownership: Ownership.any }));
    }
  }, [relaySettingsUpdater, resetScrollPositions, relaySettings]);

  return (
    <FilterChip
      aria-description={messages.pgettext('accessibility', 'Clear ownership filter')}
      onClick={onClearOwnership}
      {...props}>
      <FilterChip.Text>{ownershipFilterLabel}</FilterChip.Text>
      <FilterChip.Icon icon="cross" />
    </FilterChip>
  );
}
