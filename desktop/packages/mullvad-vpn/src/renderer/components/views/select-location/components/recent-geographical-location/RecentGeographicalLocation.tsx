import { useCallback } from 'react';

import type { GeographicalLocation } from '../../../../../features/locations/types';
import { FootnoteMiniSemiBold } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import type { ListItemProps } from '../../../../../lib/components/list-item';
import { useLocationAriaDescription, useLocationAriaLabel } from '../../hooks';
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

  const ariaLabel = useLocationAriaLabel(location);
  const ariaDescription = useLocationAriaDescription(location);

  const locationBreadcrumbs = useLocationBreadcrumbs(location);
  const breadcrumbsSubLabel = locationBreadcrumbs.join(', ');

  const disabled = location.disabled || disabledProp;

  const showParents = location.type !== 'country';

  const handleClick = useCallback(() => {
    void handleSelect(location);
  }, [location, handleSelect]);

  const showNeedsOtherEntry = location.needsOtherEntry;

  return (
    <Location root selected={location.selected}>
      <Location.ListItem disabled={disabled} level={0} position={position}>
        <Location.ListItem.Trigger
          onClick={handleClick}
          aria-label={ariaLabel}
          aria-description={ariaDescription}>
          <Location.ListItem.Item>
            <FlexColumn>
              <Location.ListItem.Item.Label>{location.label}</Location.ListItem.Item.Label>
              {showParents && (
                <FootnoteMiniSemiBold color="whiteAlpha60">
                  {breadcrumbsSubLabel}
                </FootnoteMiniSemiBold>
              )}
            </FlexColumn>
            {showNeedsOtherEntry && (
              <Location.ListItem.Item.ActionGroup>
                <Location.ListItem.Item.Icon icon="magic-multihop" />
              </Location.ListItem.Item.ActionGroup>
            )}
          </Location.ListItem.Item>
        </Location.ListItem.Trigger>
        <RecentGeographicalLocationTrailingActions location={location} />
      </Location.ListItem>
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
