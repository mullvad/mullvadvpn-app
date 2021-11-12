import React, { useCallback, useEffect } from 'react';
import { useLocation } from 'react-router';
import { useHistory } from '../lib/history';
import { disableDismissForRoutes, RoutePath } from '../lib/routes';

interface IKeyboardNavigationProps {
  children: React.ReactElement;
}

export default function KeyboardNavigation(props: IKeyboardNavigationProps) {
  const history = useHistory();
  const location = useLocation();

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        const path = location.pathname as RoutePath;
        if (!disableDismissForRoutes.includes(path)) {
          history.dismiss(true);
        }
      }
    },
    [history.reset, location.pathname],
  );

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  return props.children;
}
