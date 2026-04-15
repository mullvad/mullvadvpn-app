import React from 'react';

import { useMenuContext } from '../../../MenuContext';

export function useEffectHideOnOutsideClick() {
  const { popoverRef, triggerRef, onOpenChange } = useMenuContext();

  React.useEffect(() => {
    const handleClick = (e: PointerEvent) => {
      const popover = popoverRef?.current;
      const trigger = triggerRef?.current;
      const target = e.target as Node;

      const clickedTrigger = trigger && trigger.contains(target);
      const clickedPopover = popover && popover.contains(target);
      if (!clickedTrigger && !clickedPopover) {
        onOpenChange?.(false);
      }
    };

    document.addEventListener('pointerup', handleClick);

    return () => {
      document.removeEventListener('pointerup', handleClick);
    };
  }, [onOpenChange, popoverRef, triggerRef]);
}
