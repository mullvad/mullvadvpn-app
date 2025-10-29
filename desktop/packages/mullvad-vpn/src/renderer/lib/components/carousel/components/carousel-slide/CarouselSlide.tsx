import styled from 'styled-components';

export type CarouselSlideProps = React.ComponentPropsWithRef<'div'>;

const StyledSlide = styled.div`
  display: inline-block;
  width: 100%;
  white-space: normal;
  vertical-align: top;
  scroll-snap-align: start;
`;

export function CarouselSlide({ children, ...props }: CarouselSlideProps) {
  return (
    <StyledSlide data-carousel-slide {...props}>
      {children}
    </StyledSlide>
  );
}
