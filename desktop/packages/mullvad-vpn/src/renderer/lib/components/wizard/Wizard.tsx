import styled from 'styled-components';

import { Carousel, type CarouselProps } from '../carousel';
import { WizardControls, WizardSlides } from './components';

export type WizardProps = CarouselProps;

export const StyledWizard = styled(Carousel)`
  display: flex;
  flex: 1;
`;

function Wizard(props: WizardProps) {
  return <StyledWizard {...props}></StyledWizard>;
}

const WizardNamespace = Object.assign(Wizard, {
  Slides: WizardSlides,
  Controls: WizardControls,
});

export { WizardNamespace as Wizard };
