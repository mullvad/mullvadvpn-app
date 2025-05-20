import React from 'react';

import { RoutePath } from '../../../shared/routes';
import { useHistory } from '../../lib/history';

export const usePushAppUpgrade = () => {
  const history = useHistory();

  return React.useCallback(() => history.push(RoutePath.appUpgrade), [history]);
};
