import styled from 'styled-components';

import { spacings } from '../../../../../../foundations';
import { Carousel } from '../../../../../carousel';
import type { CarouselSlideProps } from '../../../../../carousel/components/carousel-slides/components';
import { Gallery } from '../../../../../gallery';
import { StyledGalleryTextGroup } from '../../../../../gallery/components';
import { ProgressButton } from '../../../../../progress-button';
import {
  StyledSlideTitle,
  StyledWizardSlideIcon,
  WizardSlideIcon,
  WizardSlideText,
  WizardSlideTitle,
} from './components';

export type WizardSlideProps = CarouselSlideProps;

export const StyledWizardSlide = styled(Carousel.Slides.Slide)`
  ${StyledSlideTitle}:not(:last-child) {
    margin-bottom: ${spacings.medium};
  }
  ${StyledWizardSlideIcon}:not(:last-child) {
    margin-bottom: ${spacings.large};
  }

  ${StyledGalleryTextGroup}:not(:last-child) {
    margin-bottom: ${spacings.large};
  }
`;

function WizardSlide(props: WizardSlideProps) {
  return <StyledWizardSlide {...props}></StyledWizardSlide>;
}

const WizardSlideNamespace = Object.assign(WizardSlide, {
  Title: WizardSlideTitle,
  Icon: WizardSlideIcon,
  Text: WizardSlideText,
  TextGroup: Gallery.TextGroup,
  Image: Gallery.Image,
  ProgressButton: ProgressButton,
});

export { WizardSlideNamespace as WizardSlide };
