import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../foundations';
import { useAccordionContext } from '../AccordionContext';
import { StyledAccordionHeader } from './AccordionHeader';
import { StyledAccordionIcon } from './AccordionIcon';

export type AccordionTriggerProps = {
  children?: React.ReactNode;
} & React.ButtonHTMLAttributes<HTMLButtonElement>;

const StyledAccordionTrigger = styled.button`
  background-color: transparent;
  &&:hover > ${StyledAccordionHeader} {
    background-color: ${colors.blue60};
  }
  &&:hover > ${StyledAccordionIcon} {
    background-color: ${colors.whiteAlpha60};
  }
  &&:active > ${StyledAccordionHeader} {
    background-color: ${colors.blue40};
  }
  &&:active > ${StyledAccordionIcon} {
    background-color: ${colors.whiteAlpha40};
  }
  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: -2px;
  }
`;

export function AccordionTrigger({ children }: AccordionTriggerProps) {
  const { contentId, triggerId, titleId, expanded, onExpandedChange } = useAccordionContext();

  const onClick = React.useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      e.preventDefault();
      onExpandedChange?.(!expanded);
    },
    [onExpandedChange, expanded],
  );

  return (
    <StyledAccordionTrigger
      id={triggerId}
      aria-labelledby={titleId}
      aria-controls={contentId}
      aria-expanded={expanded ? 'true' : 'false'}
      onClick={onClick}>
      {children}
    </StyledAccordionTrigger>
  );
}
