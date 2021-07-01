import React, { useCallback, useEffect } from 'react';
import { useHistory } from '../lib/history';

interface IKeyboardNavigationProps {
  children: React.ReactElement;
}

export default function KeyboardNavigation(props: IKeyboardNavigationProps) {
  const history = useHistory();

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        history.dismiss(true);
      }
    },
    [history.reset],
  );

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  return props.children;
}
