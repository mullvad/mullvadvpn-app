import React from 'react';

import { Ownership } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useSelectedEntryOrExitLocation } from '../../../../../features/locations/hooks';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';
import { useActiveOwnership } from '../../hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { useOwnershipFilterLabel } from './hooks';

export type OwnershipFilterChipProps = FilterChipProps;

export function OwnershipFilterChip(props: OwnershipFilterChipProps) {
  const relaySettings = useNormalRelaySettings();
  const { setActiveOwnership } = useActiveOwnership();
  const ownershipFilterLabel = useOwnershipFilterLabel();

  const { locationType } = useSelectLocationViewContext();
  const location = useSelectedEntryOrExitLocation(locationType);
  const inactive = location === 'any'; // Filter is not applied when `any` location is used

  const label = inactive
    ? // This line is here to prevent the following one to be moved up here by prettier
      // TRANSLATORS: Accessibility description for button removing the ownership filter.
      messages.pgettext('accessibility', 'Remove ownership filter')
    : // This line is here to prevent the following one to be moved up here by prettier
      // TRANSLATORS: Accessibility description for button to remove the ownership filter
      // TRANSLATORS: when the filter is currently not active.
      messages.pgettext('accessibility', 'Remove ownership filter (inactive)');

  const onClearOwnership = React.useCallback(async () => {
    if (relaySettings) {
      await setActiveOwnership(Ownership.any);
    }
  }, [relaySettings, setActiveOwnership]);

  return (
    <FilterChip aria-label={label} onClick={onClearOwnership} inactive={inactive} {...props}>
      <FilterChip.Text>{ownershipFilterLabel}</FilterChip.Text>
      <FilterChip.Icon icon="cross" />
    </FilterChip>
  );
}
