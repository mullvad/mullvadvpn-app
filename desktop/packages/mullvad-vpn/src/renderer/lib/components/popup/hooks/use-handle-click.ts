import React from 'react';

import { usePopupContext } from '../PopupContext';

export function useHandleClick() {
  const { popupRef } = usePopupContext();
  return React.useCallback(
    (e: React.MouseEvent<HTMLDialogElement>) => {
      const popup = popupRef.current;
      if (e.target === popup) {
        popup.requestClose();
      }
    },
    [popupRef],
  );
}
