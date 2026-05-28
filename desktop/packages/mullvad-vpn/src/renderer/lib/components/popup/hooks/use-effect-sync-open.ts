import React from 'react';

import { usePopupContext } from '../PopupContext';

export function useEffectSyncOpen() {
  const { popupRef, open, onOpenChange } = usePopupContext();
  React.useEffect(() => {
    const popup = popupRef.current;
    if (!popup || !popup.isConnected) return;

    if (open && !popup.open) {
      popup.showModal();
    } else if (!open && popup.open) {
      popup.requestClose();
    }
  }, [open, onOpenChange, popupRef]);
}
