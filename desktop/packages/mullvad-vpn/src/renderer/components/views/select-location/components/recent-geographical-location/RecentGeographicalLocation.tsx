import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import type { GeographicalLocation } from '../../../../../features/locations/types';
import { FootnoteMiniSemiBold } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { spacings } from '../../../../../lib/foundations';
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
            <Location.Accordion.Header.Trigger
              onClick={handleClick}
              aria-label={sprintf(
                // TRANSLATORS: Accessibility label for a button that connects to a location.
                // TRANSLATORS: Available placeholders:
                // TRANSLATORS: %(location)s - The name of the location that will be connected to when the button is clicked.
                messages.pgettext('accessibility', 'Connect to %(location)s'),
                {
                  location: location.label,
                },
              )}>
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
            </Location.Accordion.Header.Trigger>
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
