import React from 'react';

import { useStyledRef } from '../../utility-hooks';
import { CarouselProps } from './Carousel';

type CarouselContextextProps = {
  slideIndex: number;
  setSlideIndex: React.Dispatch<React.SetStateAction<number>>;
  numberOfSlides: number;
  slidesRef: React.RefObject<HTMLDivElement | null>;
  content: CarouselProps['content'];
};

const CarouselContextext = React.createContext<CarouselContextextProps | undefined>(undefined);

export const useCarouselContext = (): CarouselContextextProps => {
  const context = React.useContext(CarouselContextext);
  if (!context) {
    throw new Error('useCarouselContext must be used within a CarouselProvider');
  }
  return context;
};

type CarouselProviderProps = React.PropsWithChildren<CarouselContextextProps>;

export function CarouselProvider({
  content,
  children,
}: React.PropsWithChildren<Pick<CarouselProviderProps, 'content'>>) {
  const slidesRef = useStyledRef<HTMLDivElement>();
  const [slideIndex, setSlideIndex] = React.useState(0);

  return (
    <CarouselContextext.Provider
      value={{
        slideIndex,
        setSlideIndex,
        numberOfSlides: content.length,
        slidesRef,
        content,
      }}>
      {children}
    </CarouselContextext.Provider>
  );
}
