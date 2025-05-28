import { useCallback } from 'react';

import { RoutePath } from '../../../shared/routes';
import { useHistory } from '../../lib/history';

export const usePushChangelog = () => {
  const history = useHistory();
  return useCallback(() => history.push(RoutePath.changelog), [history]);
};
