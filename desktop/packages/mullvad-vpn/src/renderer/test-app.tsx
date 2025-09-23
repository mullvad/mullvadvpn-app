import React from 'react';
import { Provider } from 'react-redux';
import { Router } from 'react-router';
import { StyleSheetManager } from 'styled-components';

import ErrorBoundary from './components/ErrorBoundary';
import KeyboardNavigation from './components/KeyboardNavigation';
import { ModalContainer } from './components/Modal';
import { AppContext, useAppContext } from './context';
import { Theme } from './lib/components';
import History from './lib/history';

function Child() {
  const { value } = useAppContext();
  return <div>react renders: {value}</div>;
}

const history = new History({
  pathname: '/',
});

export function TestApp() {
  const value = React.useMemo(() => {
    return {
      app: {
        value: 'testaa',
      },
    };
  }, []);

  return (
    <div>
      <AppContext value={value}>
        <Child />
        <StyleSheetManager enableVendorPrefixes>
          <Router history={history.asHistory}>
            <Theme>
              <ErrorBoundary>
                <ModalContainer>
                  <KeyboardNavigation>
                    <div>children rendered</div>
                  </KeyboardNavigation>
                </ModalContainer>
              </ErrorBoundary>
            </Theme>
          </Router>
        </StyleSheetManager>
      </AppContext>
    </div>
  );
}
