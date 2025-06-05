import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../foundations';
import { ButtonBase } from '../../button';
import { useAccordionContext } from '../AccordionContext';
import { StyledAccordionHeader } from './AccordionHeader';

export type AccordionTriggerProps = {
  children?: React.ReactNode;
};

const StyledAccordionTrigger = styled(ButtonBase)`
  background-color: transparent;
  &&:hover > ${StyledAccordionHeader} {
    background-color: ${colors.blue60};
  }
  &&:active > ${StyledAccordionHeader} {
    background-color: ${colors.blue40};
  }
  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: -2px;
  }
`;

export function AccordionTrigger({ children }: AccordionTriggerProps) {
  const { contentId, triggerId, expanded, onExpandedChange } = useAccordionContext();

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
      aria-controls={contentId}
      aria-expanded={expanded ? 'true' : 'false'}
      onClick={onClick}>
      {children}
    </StyledAccordionTrigger>
  );
}
