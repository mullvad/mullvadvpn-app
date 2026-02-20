import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { SectionTitle } from '../../../../../lib/components/section-title';
import { spacings } from '../../../../../lib/foundations';
import { useLocationsContext } from '../../LocationsContext';
import { GeographicalLocationListItem } from '../geographical-location-list-item';
import { useLocationListsContext } from '../location-lists/LocationListsContext';
import { useRelayCount } from './hooks';

const StyledLocationContainer = styled.div`
  // Add spacing to the last child
  & > :last-child {
    margin-bottom: ${spacings.tiny};
  }
`;

export function CountryLocationList() {
  const { searchedLocations } = useLocationsContext();
  const { visibleRelays, totalRelays } = useRelayCount();

  const showFilterText = visibleRelays !== totalRelays;

  const { handleSelect } = useLocationListsContext();

  return (
    <FlexColumn gap="tiny">
      <SectionTitle>
        <SectionTitle.Title>
          {messages.pgettext('select-location-view', 'All locations')}
        </SectionTitle.Title>
        <SectionTitle.Divider />
        {showFilterText && (
          <SectionTitle.Text>
            {sprintf(
              // TRANSLATORS: Text showing how many locations are currently shown out of the total number of locations, e.g. "Showing 5 of 250"
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(visibleRelays)s: The number of relays currently shown
              // TRANSLATORS: %(totalRelays)s: The total number of relays
              messages.pgettext(
                'select-location-view',
                'Showing %(visibleRelays)s of %(totalRelays)s',
              ),
              {
                visibleRelays,
                totalRelays,
              },
            )}
          </SectionTitle.Text>
        )}
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
