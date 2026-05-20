import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../../../../../foundations';
import { StyledTextFieldInput } from '../../../../../../../text-field/components';
import { useLocationSelectorItemContext } from '../../LocationSelectorItemContext';

export type LocationSelectorTriggerProps = React.ComponentPropsWithoutRef<'div'>;

export const StyledLocationTextFieldTrigger = styled.div`
  margin: 1px;
  &:focus-visible {
    ${StyledTextFieldInput} {
      outline-color: ${colors.chalk};
      outline-width: 2px;
      outline-offset: -1px;
    }
  }
`;

export function LocationSelectorTrigger({ children, ...props }: LocationSelectorTriggerProps) {
  const { inputRef, selected, setSelected } = useLocationSelectorItemContext();

  const handleKeyDown = React.useCallback(
    (event: React.KeyboardEvent) => {
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        setSelected(true);
        inputRef.current?.focus();
      }
    },
    [inputRef, setSelected],
  );

  const locationTextFieldTriggerTabIndex = selected ? -1 : 0;
  return (
    <StyledLocationTextFieldTrigger
      tabIndex={locationTextFieldTriggerTabIndex}
      onKeyDown={handleKeyDown}
      {...props}>
      {children}
    </StyledLocationTextFieldTrigger>
  );
}
