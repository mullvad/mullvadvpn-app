import * as React from 'react';
import App from '../app';

export interface IAppReduxContext {
  app: App;
}

export const AppReduxContext = React.createContext<IAppReduxContext | undefined>(undefined);
AppReduxContext.displayName = 'AppReduxContext';

export default function withAppContext<Props>(BaseComponent: React.ComponentClass<Props>) {
  // Exclude the IAppReduxContext from props since those are injected props
  const wrappedComponent = (props: Omit<Props, keyof IAppReduxContext>) => {
    return (
      <AppReduxContext.Consumer>
        {(context) => {
          if (context) {
            // Enforce type because Typescript does not recognize that
            // (Props ~ IAppReduxContext & IAppReduxContext) is identical to Props.
            const mergedProps = ({ ...props, ...context } as unknown) as Props;

            return <BaseComponent {...mergedProps} />;
          } else {
            throw new Error(
              'The context value is empty. Make sure to wrap the component in AppReduxContext.Provider or use withAppContext',
            );
          }
        }}
      </AppReduxContext.Consumer>
    );
  };

  if (process.env.NODE_ENV === 'development') {
    wrappedComponent.displayName =
      'withAppContext(' + (BaseComponent.displayName || BaseComponent.name) + ')';
  }

  return wrappedComponent;
}
