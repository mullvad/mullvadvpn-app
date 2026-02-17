import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { SectionTitle } from '../../../../../lib/components/section-title';
import { spacings } from '../../../../../lib/foundations';
import { useLocationsContext } from '../../LocationsContext';
import { GeographicalLocationListItem } from '../geographical-location-list-item';
import { useLocationListsContext } from '../location-lists/LocationListsContext';

const StyledLocationContainer = styled.div`
  // Add spacing to the last child
  & > :last-child {
    margin-bottom: ${spacings.tiny};
  }
`;

export function CountryLocationList() {
  const { searchedLocations } = useLocationsContext();

  const { handleSelect } = useLocationListsContext();

  return (
    <FlexColumn gap="tiny">
      <SectionTitle>
        <SectionTitle.Title>
          {messages.pgettext('select-location-view', 'All locations')}
        </SectionTitle.Title>
        <SectionTitle.Divider />
      </SectionTitle>
      <FlexColumn>
        {searchedLocations.map((country) => {
          return (
            <StyledLocationContainer key={Object.values(country.details).join('-')}>
              <GeographicalLocationListItem location={country} level={0} onSelect={handleSelect} />
            </StyledLocationContainer>
          );
        })}
      </FlexColumn>
    </FlexColumn>
  );
}
