import { messages } from '../../../../../../shared/gettext';
import { LocationType } from '../../../../../features/locations/types';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { SectionTitle } from '../../../../../lib/components/section-title';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { getLocationListItemMapProps } from '../../utils';
import { CountryLocation } from '../country-location';
import { CustomListLocation } from '../custom-list-location';

export function RecentLocations() {
  const { locationType, recentLocations } = useSelectLocationViewContext();

  const locations =
    locationType === LocationType.entry ? recentLocations.entry : recentLocations.exit;

  return (
    <FlexColumn gap="tiny">
      <SectionTitle>
        <SectionTitle.Title>
          {messages.pgettext('select-location-view', 'Recents')}
        </SectionTitle.Title>
        <SectionTitle.Divider />
      </SectionTitle>
      <FlexColumn>
        {locations.map((location) => {
          const { key } = getLocationListItemMapProps(location, undefined);
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
