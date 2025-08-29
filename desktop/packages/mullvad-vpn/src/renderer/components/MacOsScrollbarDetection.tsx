import { useEffect } from 'react';
import styled from 'styled-components';

import { MacOsScrollbarVisibility } from '../../shared/ipc-schema';
import useActions from '../lib/actionsHook';
import { useEffectEvent, useStyledRef } from '../lib/utility-hooks';
import { useSelector } from '../redux/store';
import userInterface from '../redux/userinterface/actions';

const StyledContainer = styled.div({
  position: 'absolute',
  visibility: 'hidden',
  overflowY: 'scroll',
  overflowX: 'hidden',
  width: '1px',
  height: '0px',
});

// This component is used to determine whether scrollbars should be always visible or only visible
// while scrolling when the system setting for this is set to "Automatic". This is detected by
// testing if any space is taken by a scrollbar.
export default function MacOsScrollbarDetection() {
  const visibility = useSelector((state) => state.userInterface.macOsScrollbarVisibility);
  const { setMacOsScrollbarVisibility } = useActions(userInterface);
  const ref = useStyledRef<HTMLDivElement>();

  const detectVisibility = useEffectEvent((visibility?: MacOsScrollbarVisibility) => {
    if (visibility === MacOsScrollbarVisibility.automatic) {
      // If the width is 0 then the 1 px width of the parent has been used by the scrollbar.
      const newVisibility =
        ref.current?.offsetWidth === 0
          ? MacOsScrollbarVisibility.always
          : MacOsScrollbarVisibility.whenScrolling;
      setMacOsScrollbarVisibility(newVisibility);
    }
  });

  // These lint rules are disabled for now because the react plugin for eslint does
  // not understand that useEffectEvent should not be added to the dependency array.
  // Enable these rules again when eslint can lint useEffectEvent properly.
  // eslint-disable-next-line react-compiler/react-compiler
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => detectVisibility(visibility), [visibility]);

  return (
    <StyledContainer>
      <div ref={ref} />
    </StyledContainer>
  );
}
