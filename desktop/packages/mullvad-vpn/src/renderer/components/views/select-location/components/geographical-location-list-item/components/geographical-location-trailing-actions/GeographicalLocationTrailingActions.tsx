import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { AddLocationToCustomListDialog } from '../../../../../../../features/custom-lists/components';
import { useCustomLists } from '../../../../../../../features/custom-lists/hooks';
import { type GeographicalLocation } from '../../../../../../../features/locations/types';
import { getLocationChildren } from '../../../../../../../features/locations/utils';
import { useAccordionContext } from '../../../../../../../lib/components/accordion/AccordionContext';
import { LocationListItem } from '../../../location-list-item';
import { AddLocationToCustomListButton } from './components';

export type GeographicalLocationTrailingActionsProps = React.PropsWithChildren<{
  location: GeographicalLocation;
}>;

export function GeographicalLocationTrailingActions({
  location,
}: GeographicalLocationTrailingActionsProps) {
  const { customLists } = useCustomLists();
  const { expanded } = useAccordionContext();

  const [addLocationToCustomListDialogOpen, setAddLocationToCustomListDialogOpen] =
    React.useState(false);
  const handleOpenDialog = React.useCallback(() => setAddLocationToCustomListDialogOpen(true), []);

  const childLocations = getLocationChildren(location);

  const showAccordionTrigger = childLocations.length > 0;
  const showAddToCustomListButton = customLists.length > 0;

  const hasAnyTrailingAction = showAddToCustomListButton || showAccordionTrigger;

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
      {showAccordionTrigger && (
        <LocationListItem.Accordion.Header.Trigger
          aria-label={sprintf(
            expanded
              ? messages.pgettext('accessibility', 'Collapse %(location)s')
              : messages.pgettext('accessibility', 'Expand %(location)s'),
            { location: location.label },
          )}>
          <LocationListItem.Accordion.Header.TrailingActions.Action>
            <LocationListItem.Accordion.Header.TrailingActions.Action.Icon
              icon={expanded ? 'chevron-up' : 'chevron-down'}
            />
          </LocationListItem.Accordion.Header.TrailingActions.Action>
        </LocationListItem.Accordion.Header.Trigger>
      )}
    </LocationListItem.Accordion.Header.TrailingActions>
  );
}
