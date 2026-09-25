import { messages } from '../../../../../../../../shared/gettext';
import { Text } from '../../../../../../../lib/components';
import { FlexColumn } from '../../../../../../../lib/components/flex-column';
import { getLocationListItemMapProps } from '../../../../utils';
import { AutomaticLocation } from '../../../automatic-location';
import { RecentCustomListLocation } from '../../../recent-custom-list-location';
import { RecentGeographicalLocation } from '../../../recent-geographical-location';
import { useRecentLocations } from './hooks';

export function RecentLocationList() {
  const recentLocations = useRecentLocations();
  const hasRecentLocations = recentLocations.length > 0;

  return (
    <FlexColumn gap="tiny">
      {hasRecentLocations ? (
        recentLocations.map((location) => {
          if (location === 'automatic') {
            return <AutomaticLocation key="recent-automatic-location" position="solo" />;
          }
          const { key } = getLocationListItemMapProps(location);
          if (location.type === 'customList') {
            return <RecentCustomListLocation key={key} customList={location} position="solo" />;
          } else {
            return <RecentGeographicalLocation key={key} location={location} position="solo" />;
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
  );
}
