import React from 'react';

import { AccordionProps } from './Accordion';

interface AccordionContextProps {
  triggerId: string;
  contentId: string;
  expanded: AccordionProps['expanded'];
  onExpandedChange?: AccordionProps['onExpandedChange'];
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
  expanded: boolean;
  onExpandedChange?: (open: boolean) => void;
  children: React.ReactNode;
}

export function AccordionProvider({
  triggerId,
  contentId,
  expanded,
  onExpandedChange,
  children,
}: AccordionProviderProps) {
  return (
    <AccordionContext.Provider value={{ triggerId, contentId, expanded, onExpandedChange }}>
      {children}
    </AccordionContext.Provider>
  );
}
