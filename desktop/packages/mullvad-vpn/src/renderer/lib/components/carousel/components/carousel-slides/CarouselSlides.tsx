import React from 'react';
import styled from 'styled-components';

import { spacings } from '../../../../foundations';
import { useCarouselContext } from '../../CarouselContext';
import { CarouselSlide } from './components';
import { useEffectHandleSnapChange } from './hooks';

export type CarouselSlidesProps = React.ComponentPropsWithRef<'div'>;

export const StyledCarouselSlides = styled.div`
  white-space: nowrap;
  overflow: scroll hidden;
  scroll-snap-type: x mandatory;
  scroll-behavior: smooth;

  display: flex;
  width: 100%;
  gap: ${spacings.medium};

  &&::-webkit-scrollbar {
    display: none;
  }
`;

function CarouselSlides({ children, ...props }: CarouselSlidesProps) {
  const { slidesRef } = useCarouselContext();

  useEffectHandleSnapChange();

  return (
    <StyledCarouselSlides
      ref={slidesRef}
      aria-live="polite"
      aria-atomic="true"
      tabIndex={-1}
      {...props}>
      {children}
    </StyledCarouselSlides>
  );
}

const CarouselSlidesNamespace = Object.assign(CarouselSlides, {
  Slide: CarouselSlide,
});

export { CarouselSlidesNamespace as CarouselSlides };
