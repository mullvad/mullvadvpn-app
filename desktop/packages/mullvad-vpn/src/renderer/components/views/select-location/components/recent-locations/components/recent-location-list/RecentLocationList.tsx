import styled from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import { Text } from '../../../../../../../lib/components';
import { AnimatedList } from '../../../../../../../lib/components/animated-list';
import { getLocationListItemMapProps } from '../../../../utils';
import { RecentCustomListLocation } from '../../../recent-custom-list-location';
import { RecentGeographicalLocation } from '../../../recent-geographical-location';
import { useRecentLocations } from './hooks';

const StyledAnimatedList = styled(AnimatedList)`
  display: flex;
  flex-direction: column;
`;

export function RecentLocationList() {
  const recentLocations = useRecentLocations();
  const hasRecentLocations = recentLocations.length > 0;

  return (
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
  );
}
