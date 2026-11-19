import { useInView } from 'motion/react';
import React from 'react';
import { sprintf } from 'sprintf-js';
import styled, { css } from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import { durations } from '../../../../../../foundations';
import { Gallery } from '../../../../../gallery';
import { useCarouselContext } from '../../../../CarouselContext';

export type CarouselSlideProps = React.ComponentPropsWithRef<'div'>;

const StyledSlide = styled.div<{ $active: boolean }>`
  ${({ $active }) => css`
    flex: 0 0 100%;

    white-space: normal;
    vertical-align: top;
    scroll-snap-align: start;

    opacity: ${$active ? 1 : 0.25};

    transition: opacity ${durations.normal} ease-in-out;
  `}
`;

function CarouselSlide({ children, ...props }: CarouselSlideProps) {
  const ref = React.useRef<HTMLDivElement>(null);
  const id = React.useId();
  const { slides, slideIndex } = useCarouselContext();
  const isActiveSlide = slides[slideIndex]?.id === id;

  const isInView = useInView(ref, { amount: 0.5 });

  return (
    <StyledSlide
      ref={ref}
      id={id}
      tabIndex={-1}
      inert={!isActiveSlide}
      data-carousel-slide
      role="group"
      aria-roledescription="slide"
      aria-label={sprintf(
        // TRANSLATORS: Accessibility label for carousel slides.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(index)s - Will be replaced with the active slide number.
        // TRANSLATORS: %(total)s - Will be replaced with the total number of slides.
        messages.pgettext('accessibility', '%(index)s of %(total)s'),
        {
          index: slideIndex + 1,
          total: slides.length,
        },
      )}
      $active={isInView}
      {...props}>
      <Gallery>{children}</Gallery>
    </StyledSlide>
  );
}

const CarouselSlideNamespace = Object.assign(CarouselSlide, {
  Text: Gallery.Text,
  TextGroup: Gallery.TextGroup,
  Image: Gallery.Image,
});

export { CarouselSlideNamespace as CarouselSlide };
