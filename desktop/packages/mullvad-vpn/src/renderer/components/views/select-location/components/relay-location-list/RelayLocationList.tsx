import React from 'react';
import styled from 'styled-components';

import { type RelayLocation as RelayLocationType } from '../../../../../../shared/daemon-rpc-types';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { spacings } from '../../../../../lib/foundations';
import { type RelayList } from '../../select-location-types';
import { RelayLocation } from '../relay-location';

interface CommonProps {
  selectedElementRef: React.Ref<HTMLDivElement>;
  allowAddToCustomList: boolean;
  onSelect: (value: RelayLocationType) => void;
  onExpand: (location: RelayLocationType) => void;
  onCollapse: (location: RelayLocationType) => void;
  onWillExpand: (
    locationRect: DOMRect,
    expandedContentHeight: number,
    invokedByUser: boolean,
  ) => void;
  onTransitionEnd: () => void;
}

interface RelayLocationsProps extends CommonProps {
  source: RelayList;
}

const StyledLocationContainer = styled.div`
  // If the container has children, add spacing between them
  &:not(:last-child):has(> *) {
    margin-bottom: ${spacings.small};
  }
`;

export function RelayLocationList({ source, ...props }: RelayLocationsProps) {
  return (
    <FlexColumn>
      {source.map((country) => {
        return (
          <StyledLocationContainer key={getLocationKey(country.location)}>
            <RelayLocation source={country} level={0} {...props} />
          </StyledLocationContainer>
        );
      })}
    </FlexColumn>
  );
}

function getLocationKey(location: RelayLocationType): string {
  return Object.values(location).join('-');
}
