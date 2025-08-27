import React from 'react';

import { AccordionAnimation, AccordionProps } from './Accordion';

interface AccordionContextProps {
  triggerId: string;
  contentId: string;
  titleId: string;
  expanded: AccordionProps['expanded'];
  onExpandedChange?: AccordionProps['onExpandedChange'];
  disabled?: boolean;
  animation?: AccordionAnimation;
}

const AccordionContext = React.createContext<AccordionContextProps | undefined>(undefined);

export const useAccordionContext = (): AccordionContextProps => {
  const context = React.useContext(AccordionContext);
  if (!context) {
    throw new Error('useAccordionContext must be used within a AccordionProvider');
  }
  return context;
};

interface AccordionProviderProps {
  triggerId: string;
  contentId: string;
  titleId: string;
  expanded: boolean;
  onExpandedChange?: (open: boolean) => void;
  disabled?: boolean;
  animation?: AccordionAnimation;
  children: React.ReactNode;
}

export function AccordionProvider({ children, ...props }: AccordionProviderProps) {
  return <AccordionContext.Provider value={props}>{children}</AccordionContext.Provider>;
}
