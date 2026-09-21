import styled from 'styled-components';

import type { GeographicalLocation } from '../../../../../features/locations/types';
import { GeographicalLocation as GeographicalLocationComponent } from '../geographical-location';
import { useLocationListsContext } from '../location-lists/LocationListsContext';

const StyledCountryLocation = styled.div``;

export type CountryLocationProps = {
  location: GeographicalLocation;
};

export function CountryLocation({ location }: CountryLocationProps) {
  const { handleSelect } = useLocationListsContext();

  return (
    <StyledCountryLocation>
      <GeographicalLocationComponent root location={location} level={0} onSelect={handleSelect} />
    </StyledCountryLocation>
  );
}
