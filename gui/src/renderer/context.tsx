import React, { useContext } from 'react';

import App from './app';

export interface IAppContext {
  app: App;
}

export const AppContext = React.createContext<IAppContext | undefined>(undefined);
if (window.env.development) {
  AppContext.displayName = 'AppContext';
}

const missingContextError = new Error(
  'The context value is empty. Make sure to wrap the component in AppContext.Provider.',
);

export function useAppContext(): App {
  const appContext = useContext(AppContext);
  if (appContext) {
    return appContext.app;
  } else {
    throw missingContextError;
  }
}
