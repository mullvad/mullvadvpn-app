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

export function LocationSelectorTrigger({
  children,
  onBlur: onBlurProps,
  ...props
}: LocationSelectorTriggerProps) {
  const { inputRef, triggerRef, setFocusInsideTrigger } = useLocationSelectorTextFieldItemContext();
  const [tabIndex, setTabIndex] = React.useState(-1);

  const triggerFocusChange = React.useCallback(() => {
    setFocusInsideTrigger(true);
    inputRef.current?.focus();
    setTabIndex(-1);
  }, [inputRef, setFocusInsideTrigger]);

  const handleKeyDown = React.useCallback(
    (event: React.KeyboardEvent) => {
      if (document.activeElement === triggerRef.current) {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          triggerFocusChange();
        }
      }
    },
    [triggerFocusChange, triggerRef],
  );

  const handleMouseDown = React.useCallback(
    (event: React.MouseEvent<HTMLDivElement>) => {
      // Move focus to the input when trigger is interacted with, but only
      // if it is not already focused, as otherwise that would make it
      // always run its focus logic when interacted with.
      if (document.activeElement !== inputRef.current) {
        event.preventDefault();
        triggerFocusChange();
      }
    },
    [triggerFocusChange, inputRef],
  );

  React.useEffect(() => {
    setTabIndex(inputRef.current === document.activeElement ? -1 : 0);
  }, [inputRef]);

  const handleBlur = React.useCallback(
    (e: React.FocusEvent<HTMLDivElement>) => {
      const focusWillBeWithinTrigger = triggerRef.current?.contains(e.relatedTarget) ?? false;
      const focusWillBeOnTriggerElement = triggerRef.current == e.relatedTarget;

      // If the triggerRef element is the next element which will receive focus,
      // then the focus is no longer "inside" the trigger in the strictest sense.
      const focusInsideTrigger = !focusWillBeOnTriggerElement && focusWillBeWithinTrigger;

      setFocusInsideTrigger(focusInsideTrigger);
      if (!focusInsideTrigger) {
        onBlurProps?.(e);
        setTabIndex(0);
      }
    },
    [triggerRef, setFocusInsideTrigger, onBlurProps],
  );

  return (
    <StyledLocationTextFieldTrigger
      ref={triggerRef}
      tabIndex={tabIndex}
      onKeyDown={handleKeyDown}
      onMouseDown={handleMouseDown}
      onBlur={handleBlur}
      role="button"
      {...props}>
      {children}
    </StyledLocationTextFieldTrigger>
  );
}
