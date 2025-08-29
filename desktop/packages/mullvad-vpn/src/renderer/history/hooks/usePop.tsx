import React from 'react';

import { useHistory } from '../../lib/history';

export const usePop = () => {
  const history = useHistory();
  return React.useCallback(() => history.pop(), [history]);
};
