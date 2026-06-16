import React from 'react';

import { useMenuContext } from '../MenuContext';

export function useEffectSetTriggerAttributes() {
  const { triggerRef, popupId, open } = useMenuContext();
  React.useEffect(() => {
    const trigger = triggerRef.current;
    if (trigger) {
      trigger.setAttribute('aria-controls', popupId);
      trigger.setAttribute('aria-expanded', `${open}`);
      trigger.setAttribute('aria-haspopup', 'menu');
      trigger.style.setProperty('anchor-name', `--${popupId}`);
    }
  }, [popupId, open, triggerRef]);
}
