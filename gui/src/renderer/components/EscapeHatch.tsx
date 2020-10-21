import React, { useCallback, useEffect } from 'react';
import { useSelector } from 'react-redux';
import { useHistory } from 'react-router';
import { IReduxState } from '../redux/store';

interface IEscapeHatchProps {
  children: React.ReactElement;
}

export default function EscapeHatchT(props: IEscapeHatchProps) {
  const history = useHistory();
  const loggedIn = useSelector((state: IReduxState) => state.account.accountToken !== undefined);
  const connectedToDaemon = useSelector(
    (state: IReduxState) => state.userInterface.connectedToDaemon,
  );

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      const path = history.location.pathname;
      if (event.key === 'Escape' && path !== '/' && path !== '/connect' && path !== '/login') {
        if (!connectedToDaemon) {
          history.push('/');
        } else if (loggedIn) {
          history.push('/connect');
        } else {
          history.push('/login');
        }
      }
    },
    [history.push, history.location.pathname, loggedIn, connectedToDaemon],
  );

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  return props.children;
}
