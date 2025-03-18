import { useCallback } from 'react';
import { useHistory } from 'react-router';

import { RoutePath } from '../../../../../../lib/routes';

export const useHandleOnClick = () => {
  const history = useHistory();

  const handleOnClick = useCallback(() => {
    history.push(RoutePath.problemReport);
  }, [history]);

  return handleOnClick;
};
