import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { useListItemContext } from '../../../../../../../lib/components/list-item/ListItemContext';
import { LocationListItem } from '../../../../../../location-list-item';
import {
  type GeographicalLocation,
  getLocationChildrenByType,
} from '../../../../select-location-types';
import { AddToCustomListButton, RemoveFromCustomListButton } from '..';

export type GeographicalLocationTrailingActionsProps = React.PropsWithChildren<{
  location: GeographicalLocation;
}>;

export function GeographicalLocationTrailingActions({
  location,
}: GeographicalLocationTrailingActionsProps) {
  const { level } = useListItemContext();

  const childLocations = getLocationChildrenByType(location);
  const hasChildren = childLocations.some((child) => child.visible);

  const showAddToCustomListButton = location.details.customList === undefined;

  const showRemoveFromCustomListButton = location.details.customList !== undefined && level === 1;

  const hasAnyTrailingAction =
    showAddToCustomListButton || showRemoveFromCustomListButton || hasChildren;

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
      {hasChildren && (
        <LocationListItem.AccordionTrigger
          aria-label={sprintf(
            location.expanded === true
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
