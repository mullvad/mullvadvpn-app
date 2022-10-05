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

type PropsWithoutAppContext<Props> = Omit<Props, 'app'>;

export default function withAppContext<Props>(
  BaseComponent: React.ComponentType<PropsWithoutAppContext<Props> & IAppContext>,
) {
  // Exclude the IAppContext from props since those are injected props
  const wrappedComponent = (props: PropsWithoutAppContext<Props>) => {
    const appContext = useAppContext();
    return <BaseComponent app={appContext} {...props} />;
  };

  if (window.env.development) {
    wrappedComponent.displayName =
      'withAppContext(' + (BaseComponent.displayName || BaseComponent.name) + ')';
  }

  return wrappedComponent;
}

export function useAppContext(): App {
  const appContext = useContext(AppContext);
  if (appContext) {
    return appContext.app;
  } else {
    throw missingContextError;
  }
}
