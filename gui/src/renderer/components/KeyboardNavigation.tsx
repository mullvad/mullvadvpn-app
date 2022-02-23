import React, { useCallback, useContext, useEffect, useMemo, useState } from 'react';
import { useHistory } from '../lib/history';

interface IKeyboardNavigationProps {
  children: React.ReactElement;
}

// Listens for and handles keyboard shortcuts
export default function KeyboardNavigation(props: IKeyboardNavigationProps) {
  const history = useHistory();
  const [backAction, setBackAction] = useState<IBackActionConfiguration>();

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        if (event.shiftKey) {
          history.dismiss(true);
        } else {
          backAction?.action();
        }
      }
    },
    [history.dismiss, backAction],
  );

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  return <BackActionTracker registerBackAction={setBackAction}>{props.children}</BackActionTracker>;
}

type BackActionIcon = 'back' | 'close';
type BackActionFn = () => void;

interface IBackActionConfiguration {
  icon: BackActionIcon;
  action: BackActionFn;
}

interface IBackActionContext {
  parentBackAction?: IBackActionConfiguration;
  registerBackAction: (backAction: IBackActionConfiguration) => void;
  removeBackAction: (backAction: IBackActionConfiguration) => void;
}

export const BackActionContext = React.createContext<IBackActionContext>({
  registerBackAction(_backAction) {
    throw new Error('Missing BackActionContext');
  },
  removeBackAction(_backAction) {
    throw new Error('Missing BackActionContext');
  },
});

interface IBackActionProps {
  disabled?: boolean;
  icon?: BackActionIcon;
  action: BackActionFn;
  children: React.ReactNode;
}

// Component for registering back actions, e.g. navigate back or close modal. These are called
// either by pressing the back button in the navigation bar or by pressing escape.
export function BackAction(props: IBackActionProps) {
  const backActionContext = useContext(BackActionContext);
  const [childrenBackAction, setChildrenBackAction] = useState<IBackActionConfiguration>();

  const parentBackAction = useMemo<IBackActionConfiguration>(
    () => ({ icon: props.icon ?? 'back', action: props.action }),
    [props.icon, props.action],
  );
  const backActionConfiguration = childrenBackAction ?? parentBackAction;

  // Every time the action or the disabled property changes the action needs to be reregistered.
  useEffect((): (() => void) | void => {
    if (!props.disabled && backActionConfiguration) {
      backActionContext.registerBackAction(backActionConfiguration);
      return () => backActionContext.removeBackAction(backActionConfiguration);
    }
  }, [props.disabled, backActionConfiguration]);

  // Every back action keeps track of the back actions in its subtree. This makes it possible to
  // always use the action furthest down in the tree.
  return (
    <BackActionTracker
      registerBackAction={setChildrenBackAction}
      parentBackAction={parentBackAction}>
      {props.children}
    </BackActionTracker>
  );
}

interface IBackActionTracker {
  parentBackAction?: IBackActionConfiguration;
  registerBackAction: (backAction: IBackActionConfiguration | undefined) => void;
  children: React.ReactNode;
}

// This component keeps track of all registered back actions in it's subtree and reports one of them
// to it's parent.
function BackActionTracker(props: IBackActionTracker) {
  const [backActions, setBackActions] = useState<Array<IBackActionConfiguration>>([]);

  const registerBackAction = useCallback((backAction: IBackActionConfiguration) => {
    setBackActions((backActions) => [...backActions, backAction]);
  }, []);
  const removeBackAction = useCallback((backAction: IBackActionConfiguration) => {
    setBackActions((backActions) => backActions.filter((action) => action !== backAction));
  }, []);
  const backActionContext = useMemo(
    () => ({ parentBackAction: props.parentBackAction, registerBackAction, removeBackAction }),
    [backActions],
  );

  useEffect(() => props.registerBackAction(backActions.at(0)), [backActions]);

  return (
    <BackActionContext.Provider value={backActionContext}>
      {props.children}
    </BackActionContext.Provider>
  );
}
