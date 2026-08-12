import { styled } from 'styled-components';

import { createRange } from '../../../../../../utils';
import { Flex } from '../../../../../flex';
import { useCarouselContext } from '../../../../CarouselContext';
import { CarouselIndicator } from './components';

export const StyledCarouselIndicators = styled(Flex)`
  min-height: 24px;
`;

export function CarouselIndicators() {
  const { numberOfSlides, slideIndex } = useCarouselContext();

  const range = createRange(numberOfSlides);
  return (
    <StyledCarouselIndicators gap="small" alignItems="center">
      {range.map((_, i) => {
        const current = i === slideIndex;
        return <CarouselIndicator key={i} disabled={current} slideToGoTo={i} />;
      })}
    </StyledCarouselIndicators>
  );
}
