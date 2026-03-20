import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import {
  AddLocationToCustomListDialog,
  RemoveLocationFromCustomListButton,
} from '../../../../../../../features/custom-lists/components';
import { useCustomLists } from '../../../../../../../features/custom-lists/hooks';
import { type GeographicalLocation } from '../../../../../../../features/locations/types';
import { getLocationChildren } from '../../../../../../../features/locations/utils';
import { useAccordionContext } from '../../../../../../../lib/components/accordion/AccordionContext';
import { useListItemContext } from '../../../../../../../lib/components/list-item/ListItemContext';
import { useGeographicalLocationListItemContext } from '../../../geographical-location-list-item/GeographicalLocationListItemContext';
import { LocationListItem } from '../../../location-list-item';
import { useAnyLocationListItemContext } from '../../AnyLocationListItemContext';
import { AddLocationToCustomListButton } from './components';

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

  const [addLocationToCustomListDialogOpen, setAddLocationToCustomListDialogOpen] =
    React.useState(false);
  const handleOpenDialog = React.useCallback(() => setAddLocationToCustomListDialogOpen(true), []);

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
    <LocationListItem.Accordion.Header.TrailingActions>
      {showAddToCustomListButton && (
        <LocationListItem.Accordion.Header.TrailingActions.Action>
          <AddLocationToCustomListButton location={location} onClick={handleOpenDialog} />
          <AddLocationToCustomListDialog
            open={addLocationToCustomListDialogOpen}
            onOpenChange={setAddLocationToCustomListDialogOpen}
            location={location}
          />
        </LocationListItem.Accordion.Header.TrailingActions.Action>
      )}
      {/* Show remove from custom list button if location is top level item in a custom list. */}
      {showRemoveFromCustomListButton && (
        <LocationListItem.Accordion.Header.TrailingActions.Action>
          <RemoveLocationFromCustomListButton
            location={location}
            loading={loading}
            onLoadingChange={setLoading}
          />
        </LocationListItem.Accordion.Header.TrailingActions.Action>
      )}
      {showAccordionTrigger && (
        <LocationListItem.Accordion.Trigger
          aria-label={sprintf(
            expanded === true
              ? messages.pgettext('accessibility', 'Collapse %(location)s')
              : messages.pgettext('accessibility', 'Expand %(location)s'),
            { location: location.label },
          )}>
          <LocationListItem.Accordion.Header.TrailingActions.Action>
            <LocationListItem.Accordion.Header.Item.Chevron />
          </LocationListItem.Accordion.Header.TrailingActions.Action>
        </LocationListItem.Accordion.Trigger>
      )}
    </LocationListItem.Accordion.Header.TrailingActions>
  );
}
