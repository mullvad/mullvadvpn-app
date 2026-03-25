import React from 'react';

import { useMenuContext } from '../MenuContext';

export function useEffectSetTriggerAttributes() {
  const { triggerRef, popoverId, open } = useMenuContext();
  React.useEffect(() => {
    const trigger = triggerRef.current;
    if (trigger) {
      trigger.setAttribute('aria-controls', popoverId);
      trigger.setAttribute('aria-expanded', `${open}`);
      trigger.setAttribute('aria-haspopup', 'menu');
      trigger.style.setProperty('anchor-name', `--${popoverId}`);
    }
  }, [popoverId, open, triggerRef]);
}
