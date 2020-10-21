import React, { useCallback, useEffect } from 'react';
import { useHistory } from 'react-router';
import History from '../lib/history';

interface IEscapeHatchProps {
  children: React.ReactElement;
}

export default function KeyboardNavigation(props: IEscapeHatchProps) {
  const history = useHistory() as History;

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        history.reset();
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
