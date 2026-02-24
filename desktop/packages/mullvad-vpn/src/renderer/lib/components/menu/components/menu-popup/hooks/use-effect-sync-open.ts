import React from 'react';

import { useMenuContext } from '../../../MenuContext';

export function useEffectSyncOpen() {
  const { popoverRef, open, onOpenChange, triggerRef } = useMenuContext();

  React.useEffect(() => {
    const popover = popoverRef.current;
    if (!popover || !popover.isConnected || !triggerRef?.current) return;

    if (open) {
      // @ts-expect-error - showPopover does take an options object with a source property, but types are not updated yet
      popover.showPopover({ source: triggerRef.current });
    } else if (!open) {
      popover.hidePopover();
    }
  }, [open, onOpenChange, popoverRef, triggerRef]);
}
