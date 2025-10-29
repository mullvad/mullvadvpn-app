import React from 'react';

import { useStyledRef } from '../../utility-hooks';
import { getSlides } from './utils/get-slides';

type CarouselContextextProps = {
  slideIndex: number;
  setSlideIndex: React.Dispatch<React.SetStateAction<number>>;
  numberOfSlides: number;
  slidesRef: React.RefObject<HTMLDivElement | null>;
  slides: HTMLElement[];
};

const CarouselContextext = React.createContext<CarouselContextextProps | undefined>(undefined);

export const useCarouselContext = (): CarouselContextextProps => {
  const context = React.useContext(CarouselContextext);
  if (!context) {
    throw new Error('useCarouselContext must be used within a CarouselProvider');
  }
  return context;
};

type CarouselProviderProps = React.PropsWithChildren;

export function CarouselProvider({ children }: CarouselProviderProps) {
  const slidesRef = useStyledRef<HTMLDivElement>();
  const [slideIndex, setSlideIndex] = React.useState(0);
  const [slides, setSlides] = React.useState<HTMLElement[]>([]);

  React.useLayoutEffect(() => {
    setSlides(getSlides(slidesRef.current));
  }, [slidesRef]);

  return (
    <CarouselContextext.Provider
      value={{
        slideIndex,
        setSlideIndex,
        numberOfSlides: slides.length,
        slidesRef,
        slides,
      }}>
      {children}
    </CarouselContextext.Provider>
  );
}
