import React from 'react';

import {
  GeographicalLocationMenu,
  GeographicalLocationMenuButton,
} from '../../../../../../../features/locations/components';
import type { GeographicalLocation } from '../../../../../../../features/locations/types';
import { Location } from '../../../location-list-item';

export type RecentGeographicalLocationTrailingActionProps = React.PropsWithChildren<{
  location: GeographicalLocation;
}>;

export function RecentGeographicalLocationTrailingActions({
  location,
}: RecentGeographicalLocationTrailingActionProps) {
  const geographicalLocationButtonRef = React.useRef<HTMLButtonElement>(null);
  const [geographicalLocationMenuOpen, setGeographicalLocationMenuOpen] = React.useState(false);
  const toggleGeographicalLocationMenu = React.useCallback(() => {
    setGeographicalLocationMenuOpen((prev) => !prev);
  }, []);

  return (
    <Location.Accordion.Header.TrailingActions>
      <Location.Accordion.Header.TrailingActions.Action>
        <GeographicalLocationMenuButton
          ref={geographicalLocationButtonRef}
          location={location}
          onClick={toggleGeographicalLocationMenu}
        />
        <GeographicalLocationMenu
          triggerRef={geographicalLocationButtonRef}
          open={geographicalLocationMenuOpen}
          onOpenChange={setGeographicalLocationMenuOpen}
          location={location}
        />
      </Location.Accordion.Header.TrailingActions.Action>
    </Location.Accordion.Header.TrailingActions>
  );
}
