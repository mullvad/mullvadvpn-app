import React from 'react';

import { useHistory } from '../../lib/history';
import { RoutePath } from '../../lib/routes';

export const usePushAppUpgrade = () => {
  const history = useHistory();
  // TODO: Change to navigate to in app upgrade view once available
  return React.useCallback(() => history.push(RoutePath.account), [history]);
};
