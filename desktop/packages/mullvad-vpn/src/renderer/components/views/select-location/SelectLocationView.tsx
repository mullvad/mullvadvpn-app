import { useCallback } from 'react';
import React from 'react';
import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { LocationType } from '../../../features/locations/types';
import { Carousel } from '../../../lib/components/carousel';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { colors } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { HeaderMenuIconButton, SelectLocationSelector } from './components';
import { LocationSlide } from './components/location-slide';
import {
  SelectLocationViewProvider,
  useSelectLocationViewContext,
} from './SelectLocationViewContext';

const StyledStickyContainer = styled.div`
  position: sticky;
  top: 0;
  z-index: 20;
  width: 100%;
  background-color: ${colors.darkBlue};
`;

export function SelectLocationViewImpl() {
  const history = useHistory();
  const { locationType } = useSelectLocationViewContext();
  const [slideIndex, setSlideIndex] = React.useState(locationType === LocationType.entry ? 0 : 1);

  React.useLayoutEffect(() => {
    setSlideIndex(locationType === LocationType.entry ? 0 : 1);
  }, [locationType]);

  const onClose = useCallback(() => history.pop(), [history]);

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={onClose}>
        <NavigationContainer>
          <StyledStickyContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('select-location-nav', 'Select location')
              }
              titleVisible>
              <HeaderMenuIconButton />
            </AppNavigationHeader>
            <FlexColumn margin={{ horizontal: 'medium' }} padding={{ bottom: 'small' }} gap="small">
              <SelectLocationSelector />
            </FlexColumn>
          </StyledStickyContainer>
          <Carousel disableScroll slideIndex={slideIndex} onSlideIndexChange={setSlideIndex}>
            <Carousel.Slides>
              <LocationSlide type={LocationType.entry} />
              <LocationSlide type={LocationType.exit} />
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
