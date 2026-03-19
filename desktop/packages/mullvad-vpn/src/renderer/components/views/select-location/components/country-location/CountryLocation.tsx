import styled from 'styled-components';

import { type GeographicalLocation } from '../../../../../features/locations/types';
import { spacings } from '../../../../../lib/foundations';
import { GeographicalLocationListItem } from '../geographical-location-list-item';
import { useLocationListsContext } from '../location-lists/LocationListsContext';

const StyledLocationContainer = styled.div`
  margin-bottom: ${spacings.tiny};
`;

export type CountryLocationProps = {
  location: GeographicalLocation;
};

export function CountryLocation({ location }: CountryLocationProps) {
  const { handleSelect } = useLocationListsContext();

  return (
    <StyledLocationContainer>
      <GeographicalLocationListItem root location={location} level={0} onSelect={handleSelect} />
    </StyledLocationContainer>
  );
}
