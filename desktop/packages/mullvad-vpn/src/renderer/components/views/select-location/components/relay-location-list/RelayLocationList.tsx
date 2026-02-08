import React from 'react';
import styled from 'styled-components';

import { type RelayLocation as RelayLocationType } from '../../../../../../shared/daemon-rpc-types';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { spacings } from '../../../../../lib/foundations';
import { type AnyLocation } from '../../select-location-types';
import { RelayLocation } from '../relay-location';

export type RelayLocationListProps = {
  locations: AnyLocation[];
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: RelayLocationType) => void;
};

const StyledLocationContainer = styled.div`
  // If the container has children, add spacing between them
  &:not(:last-child):has(> *) {
    margin-bottom: ${spacings.small};
  }
`;

export function RelayLocationList({ locations, ...props }: RelayLocationListProps) {
  return (
    <FlexColumn>
      {locations.map((location) => {
        return (
          <StyledLocationContainer key={Object.values(location.details).join('-')}>
            <RelayLocation location={location} level={0} {...props} />
          </StyledLocationContainer>
        );
      })}
    </FlexColumn>
  );
}
