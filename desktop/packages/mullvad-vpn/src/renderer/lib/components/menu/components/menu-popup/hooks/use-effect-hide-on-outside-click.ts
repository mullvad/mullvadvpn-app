import React from 'react';

import { useMenuContext } from '../../../MenuContext';

export function useEffectHideOnOutsideClick() {
  const { popoverRef, onOpenChange } = useMenuContext();

  React.useEffect(() => {
    const handleClick = (e: PointerEvent) => {
      const popover = popoverRef?.current;
      const target = e.target as Node;

      const clickedPopover = popover && popover.contains(target);
      if (!clickedPopover) {
        onOpenChange?.(false);
      }
    };

    document.addEventListener('pointerup', handleClick);

    return () => {
      document.removeEventListener('pointerup', handleClick);
    };
  }, [onOpenChange, popoverRef]);
}
