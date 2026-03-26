import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { useCustomLists } from '../../../../../../../features/custom-lists/hooks';
import {
  GeographicalLocationMenu,
  GeographicalLocationMenuButton,
} from '../../../../../../../features/locations/components';
import { type GeographicalLocation } from '../../../../../../../features/locations/types';
import { getLocationChildren } from '../../../../../../../features/locations/utils';
import { useAccordionContext } from '../../../../../../../lib/components/accordion/AccordionContext';
import { Location } from '../../../location-list-item';

export type GeographicalLocationTrailingActionsProps = React.PropsWithChildren<{
  location: GeographicalLocation;
}>;

export function GeographicalLocationTrailingActions({
  location,
}: GeographicalLocationTrailingActionsProps) {
  const { customLists } = useCustomLists();
  const { expanded } = useAccordionContext();

  const geographicalLocationButtonRef = React.useRef<HTMLButtonElement>(null);
  const [geographicalLocationMenuOpen, setGeographicalLocationMenuOpen] = React.useState(false);
  const showGeographicalLocationMenu = React.useCallback(() => {
    setGeographicalLocationMenuOpen(true);
  }, []);

  const childLocations = getLocationChildren(location);

  const showAccordionTrigger = childLocations.length > 0;
  const showAddToCustomListButton = customLists.length > 0;

  const hasAnyTrailingAction = showAddToCustomListButton || showAccordionTrigger;

  if (!hasAnyTrailingAction) {
    return null;
  }

  return (
    <Location.Accordion.Header.TrailingActions>
      {showAddToCustomListButton && (
        <Location.Accordion.Header.TrailingActions.Action>
          <GeographicalLocationMenuButton
            ref={geographicalLocationButtonRef}
            location={location}
            onClick={showGeographicalLocationMenu}
          />
          <GeographicalLocationMenu
            triggerRef={geographicalLocationButtonRef}
            open={geographicalLocationMenuOpen}
            onOpenChange={setGeographicalLocationMenuOpen}
            location={location}
          />
        </Location.Accordion.Header.TrailingActions.Action>
      )}
      {showAccordionTrigger && (
        <Location.Accordion.Header.Trigger
          aria-label={sprintf(
            expanded
              ? messages.pgettext('accessibility', 'Collapse %(location)s')
              : messages.pgettext('accessibility', 'Expand %(location)s'),
            { location: location.label },
          )}>
          <Location.Accordion.Header.TrailingActions.Action>
            <Location.Accordion.Header.TrailingActions.Action.Icon
              icon={expanded ? 'chevron-up' : 'chevron-down'}
            />
          </Location.Accordion.Header.TrailingActions.Action>
        </Location.Accordion.Header.Trigger>
      )}
    </Location.Accordion.Header.TrailingActions>
  );
}
