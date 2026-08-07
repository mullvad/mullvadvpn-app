import styled from 'styled-components';

import { spacings } from '../../../../../../foundations';
import { Carousel } from '../../../../../carousel';
import type { CarouselSlideProps } from '../../../../../carousel/components/carousel-slides/components';
import { Gallery } from '../../../../../gallery';
import { StyledGalleryTextGroup } from '../../../../../gallery/components';
import { StyledSlideIcon, StyledSlideTitle, WizardSlideIcon, WizardSlideTitle } from './components';

export type WizardSlideProps = CarouselSlideProps;

export const StyledWizardSlide = styled(Carousel.Slides.Slide)`
  &:has(${StyledSlideIcon} + ${StyledSlideTitle}) {
    ${StyledSlideIcon} {
      margin-bottom: ${spacings.medium};
    }
  }
  &:has(${StyledSlideTitle} + ${StyledGalleryTextGroup}) {
    ${StyledSlideTitle} {
      margin-bottom: ${spacings.medium};
    }
  }
`;

function WizardSlide(props: WizardSlideProps) {
  return <StyledWizardSlide {...props}></StyledWizardSlide>;
}

const WizardSlideNamespace = Object.assign(WizardSlide, {
  Title: WizardSlideTitle,
  Icon: WizardSlideIcon,
  Text: Gallery.Text,
  TextGroup: Gallery.TextGroup,
  Image: Gallery.Image,
});

export { WizardSlideNamespace as WizardSlide };
