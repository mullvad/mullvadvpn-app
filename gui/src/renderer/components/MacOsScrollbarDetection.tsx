import React, { useEffect, useRef } from 'react';
import styled from 'styled-components';
import useActions from '../lib/actionsHook';
import userInterface from '../redux/userinterface/actions';
import { useSelector } from '../redux/store';
import { MacOsScrollbarVisibility } from '../../shared/ipc-schema';

const StyledContainer = styled.div({
  position: 'absolute',
  visibility: 'hidden',
  overflowY: 'scroll',
  overflowX: 'hidden',
  width: '1px',
  height: '0px',
});

// This component is used to determine wheter scrollbars should be always visible or only visible
// while scrolling when the system setting for this is set to "Automatic". This is detected by
// testing if any space is taken by a scrollbar.
export default function MacOsScrollbarDetection() {
  const visibility = useSelector((state) => state.userInterface.macOsScrollbarVisibility);
  const { setMacOsScrollbarVisibility } = useActions(userInterface);
  const ref = useRef() as React.RefObject<HTMLDivElement>;

  useEffect(() => {
    if (visibility === MacOsScrollbarVisibility.automatic) {
      // If the width is 0 then the 1 px width of the parent has been used by the scrollbar.
      const newVisibility =
        ref.current?.offsetWidth === 0
          ? MacOsScrollbarVisibility.always
          : MacOsScrollbarVisibility.whenScrolling;
      setMacOsScrollbarVisibility(newVisibility);
    }
  }, [visibility]);

  return (
    <StyledContainer>
      <div ref={ref} />
    </StyledContainer>
  );
}
