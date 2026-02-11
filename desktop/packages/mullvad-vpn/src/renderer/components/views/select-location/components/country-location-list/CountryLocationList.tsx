import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { SectionTitle } from '../../../../../lib/components/section-title';
import { spacings } from '../../../../../lib/foundations';
import { useRelayListContext } from '../../RelayListContext';
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

export function CountryLocationList({
  onSelect,
  selectedElementRef,
  ...props
}: CountryLocationListProps) {
  const { searchTerm } = useSelectLocationViewContext();
  const { relayList } = useRelayListContext();

  if (searchTerm !== '' && !relayList.some((location) => location.visible)) {
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
          {relayList.map((country) => {
            return (
              <StyledLocationContainer key={Object.values(country.details).join('-')}>
                <GeographicalLocationListItem
                  location={country}
                  level={0}
                  selectedElementRef={selectedElementRef}
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
