import React from 'react';

import { AccordionProps } from './Accordion';

type AccordionContextProps = Omit<AccordionProps, 'children'> & {
  headerRef: React.RefObject<HTMLDivElement | null>;
  triggerId: string;
  contentId: string;
  content: HTMLDivElement | null;
  setContent: React.Dispatch<React.SetStateAction<HTMLDivElement | null>>;
};

const AccordionContext = React.createContext<AccordionContextProps | undefined>(undefined);

export const useAccordionContext = (): AccordionContextProps => {
  const context = React.useContext(AccordionContext);
  if (!context) {
    throw new Error('useAccordionContext must be used within a AccordionProvider');
  }
  return context;
};

type AccordionProviderProps = React.PropsWithChildren<
  Pick<AccordionProps, 'titleId' | 'expanded' | 'onExpandedChange' | 'disabled'>
>;

export function AccordionProvider({
  children,
  titleId: titleIdProp,
  ...props
}: AccordionProviderProps) {
  const headerRef = React.useRef<HTMLDivElement | null>(null);
  const triggerId = React.useId();
  const contentId = React.useId();
  const titleId = React.useId();
  const [content, setContent] = React.useState<HTMLDivElement | null>(null);
  return (
    <AccordionContext.Provider
      value={{
        headerRef,
        triggerId,
        contentId,
        titleId: titleIdProp ?? titleId,
        content,
        setContent,
        ...props,
      }}>
      {children}
    </AccordionContext.Provider>
  );
}
