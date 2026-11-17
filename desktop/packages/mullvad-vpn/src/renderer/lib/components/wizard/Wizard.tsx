import styled from 'styled-components';

import { spacings } from '../../foundations';
import { Carousel, type CarouselProps } from '../carousel';
import {
  StyledWizardControls,
  StyledWizardSlides,
  WizardControls,
  WizardSlides,
} from './components';

export type WizardProps = CarouselProps;

export const StyledWizard = styled(Carousel)`
  // Spacing between slides and controls.
  > ${StyledWizardSlides} + ${StyledWizardControls} {
    margin-top: ${spacings.medium};
  }
`;

function Wizard(props: WizardProps) {
  return <StyledWizard {...props}></StyledWizard>;
}

const WizardNamespace = Object.assign(Wizard, {
  Slides: WizardSlides,
  Controls: WizardControls,
});

export { WizardNamespace as Wizard };
