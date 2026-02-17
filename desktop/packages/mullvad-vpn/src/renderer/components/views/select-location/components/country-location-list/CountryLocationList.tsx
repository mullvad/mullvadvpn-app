import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { SectionTitle } from '../../../../../lib/components/section-title';
import { spacings } from '../../../../../lib/foundations';
import { useLocationsContext } from '../../LocationsContext';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { GeographicalLocationListItem } from '../geographical-location-list-item';
import { type RelayLocationListProps } from '../relay-location-list';

export type CountryLocationListProps = Omit<RelayLocationListProps, 'locations'>;

const StyledLocationContainer = styled.div`
  // Add spacing to the last child
  & > :last-child {
    margin-bottom: ${spacings.small};
  }
`;

export function CountryLocationList({ onSelect, ...props }: CountryLocationListProps) {
  const { searchTerm } = useSelectLocationViewContext();
  const { searchedLocations } = useLocationsContext();

  if (searchTerm !== '' && searchedLocations.length === 0) {
    return null;
  } else {
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
                <GeographicalLocationListItem
                  location={country}
                  level={0}
                  onSelect={onSelect}
                  {...props}
                />
              </StyledLocationContainer>
            );
          })}
        </FlexColumn>
      </FlexColumn>
    );
  }
}
