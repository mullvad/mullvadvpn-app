import { sprintf } from 'sprintf-js';

import { useCarouselContext } from '../../CarouselContext';
import { CarouselControlGroup } from '../carousel-control-group';
import { CarouselIndicator } from '../carousel-indicator';

export type CarouselIndicatorsProps = React.ComponentPropsWithRef<'div'> & {
  ariaLabelTemplate?: string;
};

export function CarouselIndicators({ ariaLabelTemplate, ...props }: CarouselIndicatorsProps) {
  const { numberOfSlides, slideIndex } = useCarouselContext();

  return (
    <CarouselControlGroup {...props}>
      {[...Array(numberOfSlides)].map((_, i) => {
        const ariaLabel = ariaLabelTemplate
          ? sprintf(ariaLabelTemplate, { index: i + 1 })
          : undefined;
        const current = i === slideIndex;
        return (
          <CarouselIndicator key={i} disabled={current} slideToGoTo={i} aria-label={ariaLabel} />
        );
      })}
    </CarouselControlGroup>
  );
}
