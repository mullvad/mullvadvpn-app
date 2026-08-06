import styled from 'styled-components';

import { spacings } from '../../../../foundations';
import { Carousel } from '../../../carousel';
import type { CarouselControlsProps } from '../../../carousel/components';
import { StyledCarouselIndicators } from '../../../carousel/components/carousel-controls/components';
import { StyledWizardButtons, WizardButtons } from './components';

export type WizardControlsProps = CarouselControlsProps;

export const StyledWizardControls = styled.div`
  &:has(${StyledCarouselIndicators} + ${StyledWizardButtons}) {
    ${StyledCarouselIndicators} {
      margin-bottom: ${spacings.medium};
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
  Buttons: WizardButtons,
});

export { WizardControlsNamespace as WizardControls };
