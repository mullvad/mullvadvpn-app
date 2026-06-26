import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../../../../../foundations';
import { StyledTextFieldInput } from '../../../../../../../text-field/components';
import { useLocationSelectorItemContext } from '../../LocationSelectorItemContext';

export type LocationSelectorTriggerProps = React.ComponentPropsWithoutRef<'div'>;

export const StyledLocationTextFieldTrigger = styled.div`
  margin: 1px;
  display: flex;
  align-items: center;

  &:focus-visible {
    ${StyledTextFieldInput} {
      outline-color: ${colors.chalk};
      outline-width: 2px;
      outline-offset: -1px;
    }
  }
`;

export function LocationSelectorTrigger({ children, ...props }: LocationSelectorTriggerProps) {
  const { inputRef, triggerRef } = useLocationSelectorItemContext();

  const handleKeyDown = React.useCallback(
    (event: React.KeyboardEvent) => {
      if (document.activeElement !== triggerRef.current) {
        return;
      }
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        inputRef.current?.focus();
      }
    },
    [inputRef, triggerRef],
  );

  const handleMouseDown = React.useCallback(() => {
    inputRef.current?.focus();
  }, [inputRef]);

  const tabIndex = inputRef.current === document.activeElement ? -1 : 0;

  return (
    <StyledLocationTextFieldTrigger
      ref={triggerRef}
      tabIndex={tabIndex}
      onKeyDown={handleKeyDown}
      onMouseDown={handleMouseDown}
      role="button"
      {...props}>
      {children}
    </StyledLocationTextFieldTrigger>
  );
}
