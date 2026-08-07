import styled from 'styled-components';

import { Carousel } from '../../../carousel';
import type { CarouselSlidesProps } from '../../../carousel/components';
import { WizardSlide } from './components';

export type WizardSlidesProps = CarouselSlidesProps;

export const StyledWizardSlides = styled(Carousel.Slides)`
  height: 100%;
`;

function WizardSlides(props: WizardSlidesProps) {
  return <StyledWizardSlides {...props}></StyledWizardSlides>;
}

const WizardSlidesNamespace = Object.assign(WizardSlides, {
  Slide: WizardSlide,
});

export { WizardSlidesNamespace as WizardSlides };
