import { useCallback } from 'react';

import { useHistory } from '../../lib/history';
import { RoutePath } from '../../lib/routes';

export const usePushChangelog = () => {
  const history = useHistory();
  return useCallback(() => history.push(RoutePath.changelog), [history]);
};
