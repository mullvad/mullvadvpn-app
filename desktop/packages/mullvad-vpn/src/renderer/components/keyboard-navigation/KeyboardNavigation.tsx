import React, { useCallback, useContext, useEffect, useMemo, useState } from 'react';

import { useEffectEvent } from '../../lib/utility-hooks';
import { useEffectRegisterEventListener } from './hooks';

interface IKeyboardNavigationProps {
  children: React.ReactElement | Array<React.ReactElement>;
}

// Listens for and handles keyboard shortcuts
export function KeyboardNavigation(props: IKeyboardNavigationProps) {
  const [backAction, setBackActionImpl] = useState<BackActionFn>();

  // Since the backaction is now a function we need to make sure it's not called when setting the
  // state.
  const setBackAction = useCallback((backAction: BackActionFn | undefined) => {
    setBackActionImpl(() => backAction);
  }, []);

  useEffectRegisterEventListener(backAction);

  return <BackActionTracker registerBackAction={setBackAction}>{props.children}</BackActionTracker>;
}

export type BackActionFn = () => void;

interface IBackActionContext {
  parentBackAction?: BackActionFn;
  registerBackAction: (backAction: BackActionFn) => void;
  removeBackAction: (backAction: BackActionFn) => void;
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
  action: BackActionFn;
  children: React.ReactNode;
}

// Component for registering back actions, e.g. navigate back or close modal.
export function BackAction(props: IBackActionProps) {
  const { registerBackAction, removeBackAction } = useContext(BackActionContext);
  const [childrenBackAction, setChildrenBackActionImpl] = useState<BackActionFn>();

  // Since the backaction is now a function we need to make sure it's not called when setting the
  // state.
  const setChildrenBackAction = useCallback((backAction: BackActionFn | undefined) => {
    setChildrenBackActionImpl(() => backAction);
  }, []);

  // Each back action needs to be unique to make `removeBackAction` work. This is accomplished by
  // wrapping it in a callback. This was an issue since `history.pop`, which is commonly used as a
  // back action, is the same function for every component.
  const backAction = useCallback(() => {
    (childrenBackAction ?? props.action)();
  }, [props.action, childrenBackAction]);

  // Every time the action or the disabled property changes the action needs to be reregistered.
  useEffect((): (() => void) | void => {
    if (!props.disabled && backAction) {
      registerBackAction(backAction);
      return () => removeBackAction(backAction);
    }
  }, [props.disabled, backAction, registerBackAction, removeBackAction]);

  // Every back action keeps track of the back actions in its subtree. This makes it possible to
  // always use the action furthest down in the tree.
  return (
    <BackActionTracker registerBackAction={setChildrenBackAction} parentBackAction={props.action}>
      {props.children}
    </BackActionTracker>
  );
}

interface IBackActionTracker {
  parentBackAction?: BackActionFn;
  registerBackAction: (backAction: BackActionFn | undefined) => void;
  children: React.ReactNode;
}

// This component keeps track of all registered back actions in it's subtree and reports one of them
// to it's parent.
function BackActionTracker(props: IBackActionTracker) {
  const [backActions, setBackActions] = useState<Array<BackActionFn>>([]);

  const registerBackAction = useCallback((backAction: BackActionFn) => {
    setBackActions((backActions) => [...backActions, backAction]);
  }, []);
  const removeBackAction = useCallback((backAction: BackActionFn) => {
    setBackActions((backActions) => backActions.filter((action) => action !== backAction));
  }, []);
  const backActionContext = useMemo(
    () => ({ parentBackAction: props.parentBackAction, registerBackAction, removeBackAction }),
    [props.parentBackAction, registerBackAction, removeBackAction],
  );

  const registerBackActionEvent = useEffectEvent((backActions: Array<BackActionFn>) => {
    props.registerBackAction(backActions.at(0));
  });

  // These lint rules are disabled for now because the react plugin for eslint does
  // not understand that useEffectEvent should not be added to the dependency array.
  // Enable these rules again when eslint can lint useEffectEvent properly.
  // eslint-disable-next-line react-compiler/react-compiler
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => registerBackActionEvent(backActions), [backActions]);

  return (
    <BackActionContext.Provider value={backActionContext}>
      {props.children}
    </BackActionContext.Provider>
  );
}
