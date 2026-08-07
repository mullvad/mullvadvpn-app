import styled from 'styled-components';

import { Carousel } from '../../../../../carousel';
import { useSlides } from '../../../../../carousel/hooks';
import { WizardExitButton, WizardNextButton, WizardPrevButton } from './components';

export const StyledWizardButtons = styled(Carousel.Controls.ControlGroup)`
  width: 100%;
`;

export function WizardButtons() {
  const { isFirstSlide, isLastSlide } = useSlides();

  const showPrevButton = !isFirstSlide;
  const showNextButton = !isLastSlide;
  const showExitButton = isLastSlide;
  return (
    <StyledWizardButtons>
      {showPrevButton && <WizardPrevButton />}
      {showNextButton && <WizardNextButton />}
      {showExitButton && <WizardExitButton />}
    </StyledWizardButtons>
  );
}
