import React from 'react';

import { useStyledRef } from '../../utility-hooks';
import { PageSliderProps } from './Carousel';

type CarouselContextextProps = {
  pageNumber: number;
  setPageNumber: React.Dispatch<React.SetStateAction<number>>;
  numberOfPages: number;
  pageContainerRef: React.RefObject<HTMLDivElement | null>;
  content: PageSliderProps['content'];
};

const CarouselContextext = React.createContext<CarouselContextextProps | undefined>(undefined);

export const useCarouselContext = (): CarouselContextextProps => {
  const context = React.useContext(CarouselContextext);
  if (!context) {
    throw new Error('useCarouselContextext must be used within a CarouselProvider');
  }
  return context;
};

type CarouselProviderProps = React.PropsWithChildren<CarouselContextextProps>;

export function CarouselProvider({
  content,
  children,
}: React.PropsWithChildren<Pick<CarouselProviderProps, 'content'>>) {
  const pageContainerRef = useStyledRef<HTMLDivElement>();

  // A state is needed to trigger a rerender. This is needed to update the "disabled" and "$current"
  // props of the arrows and page indicators.
  const [pageNumber, setPageNumber] = React.useState(0);

  return (
    <CarouselContextext.Provider
      value={{
        pageNumber,
        setPageNumber,
        numberOfPages: content.length,
        pageContainerRef,
        content,
      }}>
      {children}
    </CarouselContextext.Provider>
  );
}
