import React from 'react';

import { Ownership } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';
import { useActiveOwnership } from '../../hooks';
import { useScrollPositionContext } from '../../ScrollPositionContext';
import { useOwnershipFilterLabel } from './hooks';

export type OwnershipFilterChipProps = FilterChipProps;

export function OwnershipFilterChip(props: OwnershipFilterChipProps) {
  const relaySettings = useNormalRelaySettings();
  const { resetScrollPositions } = useScrollPositionContext();
  const { setActiveOwnership } = useActiveOwnership();
  const ownershipFilterLabel = useOwnershipFilterLabel();

  const onClearOwnership = React.useCallback(async () => {
    resetScrollPositions();
    if (relaySettings) {
      await setActiveOwnership(Ownership.any);
    }
  }, [resetScrollPositions, relaySettings, setActiveOwnership]);

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
