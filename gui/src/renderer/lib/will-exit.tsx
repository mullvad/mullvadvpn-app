import React, { useContext } from 'react';

// This context tells its subtree if it should stop rendering or not. This is useful during
// transitions, e.g. on log out, since data might be updated which makes the disappearing view
// update a lot during the transition. There's currently no support for unpausing, which can be
// added later if needed.
const willExitContext = React.createContext<boolean>(false);

export const WillExit = willExitContext.Provider;

export function useWillExit() {
  return useContext(willExitContext);
}
