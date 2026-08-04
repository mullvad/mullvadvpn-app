import { useCallback } from 'react';
import styled from 'styled-components';

import type { GeographicalLocation } from '../../../../../features/locations/types';
import { FootnoteMiniSemiBold } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { spacings } from '../../../../../lib/foundations';
import { useLocationAriaLabel } from '../../hooks';
import { Location } from '../location-list-item';
import { useLocationListsContext } from '../location-lists/LocationListsContext';
import { RecentGeographicalLocationTrailingActions } from './components';
import { useLocationBreadcrumbs } from './hooks';
import { RecentGeographicalLocationProvider } from './RecentGeographicalLocationContext';

export type RecentGeographicalLocationProps = {
  location: GeographicalLocation;
  disabled?: boolean;
};

const StyledLocationContainer = styled.div`
  margin-bottom: ${spacings.tiny};
`;

function RecentGeographicalLocationImpl({
  location,
  disabled: disabledProp,
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
    <StyledLocationContainer>
      <Location root selected={location.selected}>
        <Location.Accordion expanded disabled={disabled}>
          <Location.Accordion.Header level={0}>
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
    </StyledLocationContainer>
  );
}

export function RecentGeographicalLocation({ ...props }: RecentGeographicalLocationProps) {
  return (
    <RecentGeographicalLocationProvider>
      <RecentGeographicalLocationImpl {...props} />
    </RecentGeographicalLocationProvider>
  );
}
