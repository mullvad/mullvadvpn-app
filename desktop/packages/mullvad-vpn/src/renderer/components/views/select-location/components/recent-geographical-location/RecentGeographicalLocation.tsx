import { useCallback } from 'react';

import type { GeographicalLocation } from '../../../../../features/locations/types';
import { FootnoteMiniSemiBold } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import type { ListItemProps } from '../../../../../lib/components/list-item';
import { useLocationAriaLabel } from '../../hooks';
import { Location } from '../location-list-item';
import { useLocationListsContext } from '../location-lists/LocationListsContext';
import { RecentGeographicalLocationTrailingActions } from './components';
import { useLocationBreadcrumbs } from './hooks';
import { RecentGeographicalLocationProvider } from './RecentGeographicalLocationContext';

export type RecentGeographicalLocationProps = {
  location: GeographicalLocation;
  disabled?: boolean;
  position?: ListItemProps['position'];
};

function RecentGeographicalLocationImpl({
  location,
  disabled: disabledProp,
  position,
}: RecentGeographicalLocationProps) {
  const { handleSelect } = useLocationListsContext();

  const ariaLabel = useLocationAriaLabel(location.label);

  const locationBreadcrumbs = useLocationBreadcrumbs(location);
  const breadcrumbsSubLabel = locationBreadcrumbs.join(', ');

  const disabled = location.disabled || disabledProp;

  const showParents = location.type !== 'country';

  const handleClick = useCallback(() => {
    void handleSelect(location);
  }, [location, handleSelect]);

  return (
    <Location root selected={location.selected}>
      <Location.Accordion expanded disabled={disabled}>
        <Location.Accordion.Header level={0} position={position}>
          <Location.Accordion.Header.ItemTrigger onClick={handleClick} aria-label={ariaLabel}>
            <Location.Accordion.Header.Item>
              <FlexColumn>
                <Location.Accordion.Header.Item.Title>
                  {location.label}
                </Location.Accordion.Header.Item.Title>
                {showParents && (
                  <FootnoteMiniSemiBold color="whiteAlpha60">
                    {breadcrumbsSubLabel}
                  </FootnoteMiniSemiBold>
                )}
              </FlexColumn>
            </Location.Accordion.Header.Item>
          </Location.Accordion.Header.ItemTrigger>
          <RecentGeographicalLocationTrailingActions location={location} />
        </Location.Accordion.Header>
      </Location.Accordion>
    </Location>
  );
}

export function RecentGeographicalLocation({ ...props }: RecentGeographicalLocationProps) {
  return (
    <RecentGeographicalLocationProvider>
      <RecentGeographicalLocationImpl {...props} />
    </RecentGeographicalLocationProvider>
  );
}
