import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import {
  AddLocationToCustomListButton,
  RemoveLocationFromCustomListButton,
} from '../../../../../../../features/custom-lists/components';
import { useCustomLists } from '../../../../../../../features/custom-lists/hooks';
import { type GeographicalLocation } from '../../../../../../../features/locations/types';
import { getLocationChildren } from '../../../../../../../features/locations/utils';
import { useAccordionContext } from '../../../../../../../lib/components/accordion/AccordionContext';
import { useListItemContext } from '../../../../../../../lib/components/list-item/ListItemContext';
import { LocationListItem } from '../../../../../../location-list-item';
import { useGeographicalLocationListItemContext } from '../../../geographical-location-list-item/GeographicalLocationListItemContext';
import { useAnyLocationListItemContext } from '../../AnyLocationListItemContext';

export type GeographicalLocationTrailingActionsProps = React.PropsWithChildren<{
  location: GeographicalLocation;
}>;

export function GeographicalLocationTrailingActions({
  location,
}: GeographicalLocationTrailingActionsProps) {
  const { rootLocation } = useAnyLocationListItemContext();
  const { customLists } = useCustomLists();
  const { level } = useListItemContext();
  const { expanded } = useAccordionContext();
  const { loading, setLoading } = useGeographicalLocationListItemContext();

  const childLocations = getLocationChildren(location);

  const showAccordionTrigger = childLocations.length > 0;
  const showAddToCustomListButton = rootLocation === 'geographical' && customLists.length > 0;
  const showRemoveFromCustomListButton = rootLocation === 'customList' && level === 1;

  const hasAnyTrailingAction =
    showAddToCustomListButton || showRemoveFromCustomListButton || showAccordionTrigger;

  if (!hasAnyTrailingAction) {
    return null;
  }

  return (
    <LocationListItem.HeaderTrailingActions>
      {showAddToCustomListButton && (
        <LocationListItem.HeaderTrailingAction>
          <AddLocationToCustomListButton location={location} />
        </LocationListItem.HeaderTrailingAction>
      )}
      {/* Show remove from custom list button if location is top level item in a custom list. */}
      {showRemoveFromCustomListButton && (
        <LocationListItem.HeaderTrailingAction>
          <RemoveLocationFromCustomListButton
            location={location}
            loading={loading}
            onLoadingChange={setLoading}
          />
        </LocationListItem.HeaderTrailingAction>
      )}
      {showAccordionTrigger && (
        <LocationListItem.AccordionTrigger
          aria-label={sprintf(
            expanded === true
              ? messages.pgettext('accessibility', 'Collapse %(location)s')
              : messages.pgettext('accessibility', 'Expand %(location)s'),
            { location: location.label },
          )}>
          <LocationListItem.HeaderTrailingAction>
            <LocationListItem.HeaderChevron />
          </LocationListItem.HeaderTrailingAction>
        </LocationListItem.AccordionTrigger>
      )}
    </LocationListItem.HeaderTrailingActions>
  );
}
