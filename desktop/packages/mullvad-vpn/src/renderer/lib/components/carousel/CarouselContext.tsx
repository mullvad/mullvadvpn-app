import React from 'react';

import { useStyledRef } from '../../utility-hooks';
import { getSlides } from './utils/get-slides';

type CarouselContextextProps = {
  slideIndex: number;
  setSlideIndex: React.Dispatch<React.SetStateAction<number>>;
  numberOfSlides: number;
  carouselRef: React.RefObject<HTMLDivElement | null>;
  slidesRef: React.RefObject<HTMLDivElement | null>;
  nextButtonRef?: React.RefObject<HTMLButtonElement | null>;
  prevButtonRef?: React.RefObject<HTMLButtonElement | null>;
  firstIndicatorRef?: React.RefObject<HTMLButtonElement | null>;
  lastIndicatorRef?: React.RefObject<HTMLButtonElement | null>;
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
  const carouselRef = React.useRef<HTMLDivElement | null>(null);
  const slidesRef = useStyledRef<HTMLDivElement>();
  const nextButtonRef = React.useRef<HTMLButtonElement | null>(null);
  const prevButtonRef = React.useRef<HTMLButtonElement | null>(null);
  const firstIndicatorRef = React.useRef<HTMLButtonElement | null>(null);
  const lastIndicatorRef = React.useRef<HTMLButtonElement | null>(null);
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
        carouselRef,
        slidesRef,
        nextButtonRef,
        prevButtonRef,
        firstIndicatorRef,
        lastIndicatorRef,
        slides,
      }}>
      {children}
    </CarouselContextext.Provider>
  );
}
