import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../shared/gettext';
import {
  CustomListGeographicalLocationMenu,
  CustomListGeographicalLocationMenuButton,
} from '../../../../../../features/custom-lists/components';
import { getLocationChildren } from '../../../../../../features/locations/utils';
import { useAccordionContext } from '../../../../../../lib/components/accordion/AccordionContext';
import type { ListItemTrailingActionsProps } from '../../../../../../lib/components/list-item/components';
import { Location } from '../../location-list-item';
import { useCustomListGeographicalLocationContext } from '../CustomListGeographicalLocationContext';

export type CustomListGeographicalLocationTrailingActionsProps = ListItemTrailingActionsProps;

export function CustomListGeographicalLocationTrailingActions(
  props: CustomListGeographicalLocationTrailingActionsProps,
) {
  const { expanded } = useAccordionContext();
  const { loading, setLoading, location, level } = useCustomListGeographicalLocationContext();
  const [menuOpen, setMenuOpen] = React.useState(false);
  const toggleMenu = React.useCallback(() => {
    setMenuOpen((prev) => !prev);
  }, []);

  const locationChildren = getLocationChildren(location);
  const showAccordionTrigger = locationChildren.length > 0;
  // Show remove from custom list button if location is top level item in a custom list.
  const showMenu = level === 1;

  const triggerRef = React.useRef<HTMLButtonElement>(null);

  return (
    <Location.Accordion.Header.TrailingActions {...props}>
      {showMenu && (
        <Location.Accordion.Header.TrailingActions.Action>
          <CustomListGeographicalLocationMenuButton
            ref={triggerRef}
            location={location}
            onClick={toggleMenu}
            disabled={loading}
          />
          <CustomListGeographicalLocationMenu
            triggerRef={triggerRef}
            location={location}
            loading={loading}
            setLoading={setLoading}
            open={menuOpen}
            onOpenChange={setMenuOpen}
          />
        </Location.Accordion.Header.TrailingActions.Action>
      )}
      {showAccordionTrigger && (
        <Location.Accordion.Trigger
          aria-label={sprintf(
            expanded
              ? messages.pgettext('accessibility', 'Collapse %(location)s')
              : messages.pgettext('accessibility', 'Expand %(location)s'),
            { location: location.label },
          )}>
          <Location.Accordion.Header.TrailingActions.Action>
            <Location.Accordion.Header.TrailingActions.Action.Chevron />
          </Location.Accordion.Header.TrailingActions.Action>
        </Location.Accordion.Trigger>
      )}
    </Location.Accordion.Header.TrailingActions>
  );
}
