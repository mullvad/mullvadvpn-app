import React from 'react';

export function useIsUpdatedCustomListNameValid() {
  return React.useCallback((name: string): boolean | string => {
    const nameIsNotEmpty = name.trim() !== '';
    if (!nameIsNotEmpty) {
      return false;
    }

    return true;
  }, []);
}
