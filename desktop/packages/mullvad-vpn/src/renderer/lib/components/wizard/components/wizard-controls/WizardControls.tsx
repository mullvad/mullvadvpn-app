import styled from 'styled-components';

import { spacings } from '../../../../foundations';
import { Carousel } from '../../../carousel';
import type { CarouselControlsProps } from '../../../carousel/components';
import { StyledCarouselIndicators } from '../../../carousel/components/carousel-controls/components';
import {
  StyledWizardButtonGroup,
  WizardButtonGroup,
  WizardExitButton,
  WizardNextButton,
  WizardPrevButton,
} from './components';

export type WizardControlsProps = CarouselControlsProps;

export const StyledWizardControls = styled.div`
  &:has(${StyledCarouselIndicators} + ${StyledWizardButtonGroup}) {
    ${StyledCarouselIndicators} {
      margin-bottom: ${spacings.small};
    }
  }

  & > {
    ${StyledCarouselIndicators} {
      justify-self: center;
    }
  }
`;

function WizardControls(props: WizardControlsProps) {
  return <StyledWizardControls {...props}></StyledWizardControls>;
}

const WizardControlsNamespace = Object.assign(WizardControls, {
  Indicators: Carousel.Controls.Indicators,
  ButtonGroup: WizardButtonGroup,
  ExitButton: WizardExitButton,
  NextButton: WizardNextButton,
  PrevButton: WizardPrevButton,
});

export { WizardControlsNamespace as WizardControls };
