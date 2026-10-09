import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../../../../../foundations';
import { StyledTextFieldInput } from '../../../../../../../text-field/components/text-field-input-group/components/text-field-input';
import { useLocationSelectorTextFieldItemContext } from '../../LocationSelectorTextFieldItemContext';

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
  const { inputRef, triggerRef } = useLocationSelectorTextFieldItemContext();
  const [tabIndex, setTabIndex] = React.useState(-1);

  const handleKeyDown = React.useCallback(
    (event: React.KeyboardEvent) => {
      if (document.activeElement !== triggerRef.current) {
        return;
      }
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        inputRef.current?.focus();
        setTabIndex(-1);
      }
    },
    [inputRef, triggerRef],
  );

  const handleMouseDown = React.useCallback(
    (event: React.MouseEvent<HTMLDivElement>) => {
      // Move focus to the input when trigger is clicked, if it does not already
      // have the focus, as that would make it always run its focus logic on
      // mouse clicks.
      if (document.activeElement !== inputRef.current) {
        event.preventDefault();
        inputRef.current?.focus();
        setTabIndex(-1);
      }
    },
    [inputRef],
  );

  React.useEffect(() => {
    setTabIndex(inputRef.current === document.activeElement ? -1 : 0);
  }, [inputRef]);

  const handleOnBlur = React.useCallback(() => {
    setTabIndex(0);
  }, []);

  return (
    <StyledLocationTextFieldTrigger
      ref={triggerRef}
      tabIndex={tabIndex}
      onKeyDown={handleKeyDown}
      onMouseDown={handleMouseDown}
      onBlur={handleOnBlur}
      role="button"
      {...props}>
      {children}
    </StyledLocationTextFieldTrigger>
  );
}
