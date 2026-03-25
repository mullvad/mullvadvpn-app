import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { SectionTitle } from '../../../../../lib/components/section-title';
import { getLocationListItemMapProps } from '../../utils';
import { CountryLocation } from '../country-location';
import { CustomListLocation } from '../custom-list-location';
import { useRecentLocations } from './hooks';

export function RecentLocations() {
  const recentLocations = useRecentLocations();

  return (
    <FlexColumn gap="tiny" margin={{ bottom: 'large' }}>
      <SectionTitle>
        <SectionTitle.Title>
          {
            // TRANSLATORS: Title for section showing recently used locations.
            messages.pgettext('select-location-view', 'Recents')
          }
        </SectionTitle.Title>
        <SectionTitle.Divider />
      </SectionTitle>
      <FlexColumn>
        {recentLocations.map((location) => {
          const { key } = getLocationListItemMapProps(location);
          if (location.type === 'customList') {
            return <CustomListLocation key={key} customList={location} />;
          } else {
            return <CountryLocation key={key} location={location} />;
          }
        })}
      </FlexColumn>
    </FlexColumn>
  );
}
