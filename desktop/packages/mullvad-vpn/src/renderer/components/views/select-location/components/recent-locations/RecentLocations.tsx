import React from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { Text } from '../../../../../lib/components';
import { AnimatedList } from '../../../../../lib/components/animated-list';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { SectionTitle } from '../../../../../lib/components/section-title';
import { getLocationListItemMapProps } from '../../utils';
import { RecentCustomListLocation } from '../recent-custom-list-location';
import { RecentGeographicalLocation } from '../recent-geographical-location';
import { useRecentLocations } from './hooks';

const StyledAnimatedList = styled(AnimatedList)`
  display: flex;
  flex-direction: column;
`;

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
      <StyledAnimatedList>
        {hasRecentLocations ? (
          recentLocations.map((location) => {
            const { key } = getLocationListItemMapProps(location);
            if (location.type === 'customList') {
              return (
                <AnimatedList.Item key={key}>
                  <RecentCustomListLocation customList={location} />
                </AnimatedList.Item>
              );
            } else {
              return (
                <AnimatedList.Item key={key}>
                  <RecentGeographicalLocation location={location} />
                </AnimatedList.Item>
              );
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
      </StyledAnimatedList>
    </FlexColumn>
  );
}
