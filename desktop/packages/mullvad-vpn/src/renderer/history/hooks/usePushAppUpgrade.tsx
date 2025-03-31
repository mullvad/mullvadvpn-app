import React from 'react';

import { useHistory } from '../../lib/history';
import { RoutePath } from '../../lib/routes';

export const usePushAppUpgrade = () => {
  const history = useHistory();

  return React.useCallback(() => history.push(RoutePath.appUpgrade), [history]);
};
