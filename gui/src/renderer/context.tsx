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

export default function withAppContext<Props>(BaseComponent: React.ComponentType<Props>) {
  // Exclude the IAppContext from props since those are injected props
  const wrappedComponent = (props: Omit<Props, keyof IAppContext>) => {
    return (
      <AppContext.Consumer>
        {(context) => {
          if (context) {
            // Enforce type because Typescript does not recognize that
            // (Props ~ IAppContext & IAppContext) is identical to Props.
            const mergedProps = ({ ...props, ...context } as unknown) as Props;

            return <BaseComponent {...mergedProps} />;
          } else {
            throw missingContextError;
          }
        }}
      </AppContext.Consumer>
    );
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
