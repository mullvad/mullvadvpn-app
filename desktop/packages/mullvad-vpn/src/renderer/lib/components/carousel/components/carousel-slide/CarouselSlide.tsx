import styled from 'styled-components';

export type CarouselSlideProps = React.ComponentPropsWithRef<'div'>;

const SLIDE_GAP = 16;

const StyledSlide = styled.div({
  display: 'inline-block',
  width: '100%',
  whiteSpace: 'normal',
  verticalAlign: 'top',
  scrollSnapAlign: 'start',

  '&&:not(:last-child)': {
    marginRight: `${SLIDE_GAP}px`,
  },
});

export function CarouselSlide({ children, ...props }: CarouselSlideProps) {
  return (
    <StyledSlide data-carousel-slide {...props}>
      {children}
    </StyledSlide>
  );
}
