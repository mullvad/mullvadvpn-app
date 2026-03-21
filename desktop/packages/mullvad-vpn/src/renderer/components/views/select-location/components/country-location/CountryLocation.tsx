import styled from 'styled-components';

import type { GeographicalLocation } from '../../../../../features/locations/types';
import { spacings } from '../../../../../lib/foundations';
import { GeographicalLocation as GeographicalLocationComponent } from '../geographical-location';
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
      <GeographicalLocationComponent root location={location} level={0} onSelect={handleSelect} />
    </StyledLocationContainer>
  );
}
