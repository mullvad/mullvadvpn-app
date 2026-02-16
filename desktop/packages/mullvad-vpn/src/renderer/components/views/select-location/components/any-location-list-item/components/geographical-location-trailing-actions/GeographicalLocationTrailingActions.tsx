import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { useAccordionContext } from '../../../../../../../lib/components/accordion/AccordionContext';
import { useListItemContext } from '../../../../../../../lib/components/list-item/ListItemContext';
import { LocationListItem } from '../../../../../../location-list-item';
import {
  type GeographicalLocation,
  getLocationChildrenByType,
} from '../../../../select-location-types';
import { useAnyLocationListItemContext } from '../../AnyLocationListItemContext';
import { AddToCustomListButton, RemoveFromCustomListButton } from '..';

export type GeographicalLocationTrailingActionsProps = React.PropsWithChildren<{
  location: GeographicalLocation;
}>;

export function GeographicalLocationTrailingActions({
  location,
}: GeographicalLocationTrailingActionsProps) {
  const { rootLocation } = useAnyLocationListItemContext();
  const { level } = useListItemContext();
  const { expanded } = useAccordionContext();

  const childLocations = getLocationChildrenByType(location);

  const showAccordionTrigger = childLocations.length > 0;
  const showAddToCustomListButton = rootLocation === 'geographical';
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
          <AddToCustomListButton location={location} />
        </LocationListItem.HeaderTrailingAction>
      )}
      {/* Show remove from custom list button if location is top level item in a custom list. */}
      {showRemoveFromCustomListButton && (
        <LocationListItem.HeaderTrailingAction>
          <RemoveFromCustomListButton location={location} />
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
            <LocationListItem.Icon />
          </LocationListItem.HeaderTrailingAction>
        </LocationListItem.AccordionTrigger>
      )}
    </LocationListItem.HeaderTrailingActions>
  );
}
