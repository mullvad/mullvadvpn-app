import React from 'react';

import { BackActionFn } from '../KeyboardNavigation';
import { useHandleKeyDown } from './use-handle-key-down';

export function useEffectRegisterEventListener(backAction: BackActionFn | undefined) {
  const handleKeyDown = useHandleKeyDown(backAction);

  React.useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
}
