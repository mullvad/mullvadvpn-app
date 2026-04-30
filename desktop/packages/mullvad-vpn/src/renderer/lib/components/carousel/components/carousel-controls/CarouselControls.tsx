import styled from 'styled-components';

import {
  CarouselControlGroup,
  CarouselIndicators,
  CarouselNextButton,
  CarouselPrevButton,
} from './components';

export type CarouselControlsProps = React.ComponentPropsWithRef<'div'>;

const StyledGrid = styled.div`
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;

  && > :first-child {
    justify-self: start;
  }

  && > :last-child {
    justify-self: end;
  }
`;

function CarouselControls({ children, ...props }: CarouselControlsProps) {
  return (
    <StyledGrid {...props}>
      <div>{/* spacer to make slide indicators centered */}</div>
      {children}
    </StyledGrid>
  );
}

const CarouselControlsNamespace = Object.assign(CarouselControls, {
  ControlGroup: CarouselControlGroup,
  NextButton: CarouselNextButton,
  PrevButton: CarouselPrevButton,
  Indicators: CarouselIndicators,
});

export { CarouselControlsNamespace as CarouselControls };
