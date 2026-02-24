import React from 'react';

import { useMenuContext } from '../MenuContext';

export function useEffectSetTriggerAttributes() {
  const { triggerRef, popoverId, open } = useMenuContext();
  React.useEffect(() => {
    const trigger = triggerRef.current;
    if (trigger) {
      trigger.setAttribute('aria-controls', popoverId);
      trigger.setAttribute('aria-expanded', open ? 'true' : 'false');
    }
  }, [popoverId, open, triggerRef]);
}
