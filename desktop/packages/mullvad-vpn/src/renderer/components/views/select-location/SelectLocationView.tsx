import { useCallback } from 'react';
import React from 'react';
import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { LocationType } from '../../../features/locations/types';
import { useMeasure } from '../../../hooks';
import { Carousel } from '../../../lib/components/carousel';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { colors, spacings } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { HeaderMenuIconButton, SelectLocationSelector } from './components';
import { LocationSlide } from './components/location-slide';
import { useIsLocationSelectorExpanded } from './hooks';
import {
  SelectLocationViewProvider,
  useSelectLocationViewContext,
} from './SelectLocationViewContext';

const StyledLocationSelectorContainerWrapper = styled.div`
  position: relative;
  top: 0;
  z-index: 100;
  width: 100%;
  background-color: ${colors.darkBlue};
`;

const StyledLocationSelectorContainerSticky = styled.div`
  position. sticky;
  top: 0;
  width: 100%;
`;

const StyledLocationSelectorContainer = styled.div`
  padding-bottom: ${spacings.small};
  position: absolute;
  left: 0;
  top: 0;
  width: 100%;
  width: 100%;
  z-index: 100;
  background-color: ${colors.darkBlue};
`;

const StyledLocationSelectorMeasurementsContainer = styled.div`
  position: fixed;
  visibility: hidden;
  transform: translateX(10%);
  width: 100vw;
  left: 0;
  top: 0;
  z-index: 1000;
`;

export function SelectLocationViewImpl() {
  const history = useHistory();
  const { locationType } = useSelectLocationViewContext();
  const [slideIndex, setSlideIndex] = React.useState(locationType === LocationType.entry ? 0 : 1);

  React.useLayoutEffect(() => {
    setSlideIndex(locationType === LocationType.entry ? 0 : 1);
  }, [locationType]);

  const onClose = useCallback(() => history.pop(), [history]);

  const [expandedHeaderHeightRef, { height: expandedHeaderHeight }] = useMeasure();
  const [collapsedHeaderHeightRef, { height: collapsedHeaderHeight }] = useMeasure();

  const expanded = useIsLocationSelectorExpanded();

  return (
    <View backgroundColor="darkBlue">
      <StyledLocationSelectorMeasurementsContainer ref={expandedHeaderHeightRef}>
        <SelectLocationSelector expanded={true} />
      </StyledLocationSelectorMeasurementsContainer>
      <StyledLocationSelectorMeasurementsContainer ref={collapsedHeaderHeightRef}>
        <SelectLocationSelector expanded={false} />
      </StyledLocationSelectorMeasurementsContainer>
      <BackAction action={onClose}>
        <NavigationContainer>
          <StyledLocationSelectorContainerSticky>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('select-location-nav', 'Select location')
              }
              titleVisible>
              <HeaderMenuIconButton />
            </AppNavigationHeader>
            <StyledLocationSelectorContainerWrapper>
              <StyledLocationSelectorContainer>
                <FlexColumn margin={{ horizontal: 'medium' }}>
                  <SelectLocationSelector expanded={expanded} />
                </FlexColumn>
              </StyledLocationSelectorContainer>
            </StyledLocationSelectorContainerWrapper>
          </StyledLocationSelectorContainerSticky>
          <Carousel disableScroll slideIndex={slideIndex} onSlideIndexChange={setSlideIndex}>
            <Carousel.Slides>
              <LocationSlide
                index={0}
                type={LocationType.entry}
                collapsedHeaderHeight={collapsedHeaderHeight}
                expandedHeaderHeight={expandedHeaderHeight}
              />
              <LocationSlide
                index={1}
                type={LocationType.exit}
                collapsedHeaderHeight={collapsedHeaderHeight}
                expandedHeaderHeight={expandedHeaderHeight}
              />
            </Carousel.Slides>
          </Carousel>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

export function SelectLocationView() {
  return (
    <SelectLocationViewProvider>
      <SelectLocationViewImpl />
    </SelectLocationViewProvider>
  );
}
