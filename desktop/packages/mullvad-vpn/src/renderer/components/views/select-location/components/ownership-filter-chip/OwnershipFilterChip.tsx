import React from 'react';

import { Ownership } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useOwnership } from '../../../../../features/location/hooks';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';
import { useScrollPositionContext } from '../../ScrollPositionContext';
import { useOwnershipFilterLabel } from './hooks';

export type OwnershipFilterChipProps = FilterChipProps;

export function OwnershipFilterChip(props: OwnershipFilterChipProps) {
  const relaySettings = useNormalRelaySettings();
  const { resetScrollPositions } = useScrollPositionContext();
  const { setOwnership } = useOwnership();
  const ownershipFilterLabel = useOwnershipFilterLabel();

  const onClearOwnership = React.useCallback(async () => {
    resetScrollPositions();
    if (relaySettings) {
      await setOwnership(Ownership.any);
    }
  }, [setOwnership, resetScrollPositions, relaySettings]);

  return (
    <FilterChip
      aria-description={
        // TRANSLATORS: Accessibility description for button clearing the ownership filter.
        messages.pgettext('accessibility', 'Clear ownership filter')
      }
      onClick={onClearOwnership}
      {...props}>
      <FilterChip.Text>{ownershipFilterLabel}</FilterChip.Text>
      <FilterChip.Icon icon="cross" />
    </FilterChip>
  );
}
