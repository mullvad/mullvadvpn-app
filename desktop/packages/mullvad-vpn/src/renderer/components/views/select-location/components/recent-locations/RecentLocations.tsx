import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { Text } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { SectionTitle } from '../../../../../lib/components/section-title';
import { getLocationListItemMapProps } from '../../utils';
import { RecentCustomListLocation } from '../recent-custom-list-location';
import { RecentGeographicalLocation } from '../recent-geographical-location';
import { useRecentLocations } from './hooks';

export function RecentLocations() {
  const recentLocations = useRecentLocations();
  const titleId = React.useId();

  const hasRecentLocations = recentLocations.length > 0;

  return (
    <FlexColumn as="section" gap="tiny" margin={{ bottom: 'large' }} aria-labelledby={titleId}>
      <SectionTitle>
        <SectionTitle.Title as="h3" id={titleId}>
          {
            // TRANSLATORS: Title for section showing recently used locations.
            messages.pgettext('select-location-view', 'Recents')
          }
        </SectionTitle.Title>
        <SectionTitle.Divider />
      </SectionTitle>
      <FlexColumn>
        {hasRecentLocations ? (
          recentLocations.map((location) => {
            const { key } = getLocationListItemMapProps(location);
            if (location.type === 'customList') {
              return <RecentCustomListLocation key={key} customList={location} />;
            } else {
              return <RecentGeographicalLocation key={key} location={location} />;
            }
          })
        ) : (
          <Text variant="labelTiny" color="whiteAlpha60">
            {
              // TRANSLATORS: Message shown when the user has no recent locations.
              messages.pgettext('select-location-view', 'No recent selection history')
            }
          </Text>
        )}
      </FlexColumn>
    </FlexColumn>
  );
}
