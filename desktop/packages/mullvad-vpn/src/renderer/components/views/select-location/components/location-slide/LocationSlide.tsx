import React from 'react';
import styled from 'styled-components';

import { LocationType } from '../../../../../features/locations/types';
import { Carousel } from '../../../../../lib/components/carousel';
import { View } from '../../../../../lib/components/view';
import type { IScrollEvent } from '../../../../CustomScrollbars';
import { NavigationScrollbars } from '../../../../NavigationScrollbars';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { LocationLists } from '../location-lists';
import { SpacePreAllocationView } from '../space-pre-allocation-view';
import { LocationSlideContextProvider, useLocationSlideContext } from './LocationSlideContext';

export type LocationSlideProps = {
  type: LocationType;
};

const StyledCarouselSlide = styled(Carousel.Slides.Slide)`
  // TODO: Replace 182 with real calculated height of header
  height: calc(100vh - 182px);
`;

function LocationSlideImpl({ type }: LocationSlideProps) {
  const { spacePreAllocationViewRef, scrollViewRef } = useLocationSlideContext();
  const { setScrollTop } = useSelectLocationViewContext();

  const handleScroll = React.useCallback(
    (event: IScrollEvent) => {
      setScrollTop(event.scrollTop);
    },
    [setScrollTop],
  );

  return (
    <StyledCarouselSlide>
      <NavigationScrollbars onScroll={handleScroll} ref={scrollViewRef}>
        <SpacePreAllocationView ref={spacePreAllocationViewRef}>
          <View.Container horizontalMargin="medium" flexDirection="column">
            <LocationLists type={type} />
          </View.Container>
        </SpacePreAllocationView>
      </NavigationScrollbars>
    </StyledCarouselSlide>
  );
}

export function LocationSlide({ type }: LocationSlideProps) {
  return (
    <LocationSlideContextProvider>
      <LocationSlideImpl type={type} />
    </LocationSlideContextProvider>
  );
}
