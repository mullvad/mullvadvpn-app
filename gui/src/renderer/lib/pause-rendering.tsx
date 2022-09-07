import React, { useCallback, useContext } from 'react';

const pauseRenderingContext = React.createContext<boolean>(false);

interface PauseRenderingProps {
  pause?: boolean;
  children?: React.ReactNode;
}

// This component tells its subtree if it should pause rendering or not. This is useful during
// transitions, e.g. on log out, since data might be updated which makes the disappearing view
// update a lot during the transition. There's currently no support for unpausing, which can be
// added later if needed.
export function PauseRendering(props: PauseRenderingProps) {
  return (
    <pauseRenderingContext.Provider value={props.pause === true}>
      {props.children}
    </pauseRenderingContext.Provider>
  );
}

export function usePause() {
  const paused = useContext(pauseRenderingContext);

  const runIfNotPaused = useCallback(
    (fn: () => void) => {
      if (!paused) {
        fn();
      }
    },
    [paused],
  );

  return [paused, runIfNotPaused] as const;
}
