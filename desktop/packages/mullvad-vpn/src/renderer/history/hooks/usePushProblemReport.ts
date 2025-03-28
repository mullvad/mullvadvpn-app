import { useCallback } from 'react';
import { useHistory } from 'react-router';

import { RoutePath } from '../../lib/routes';

export const usePushProblemReport = () => {
  const history = useHistory();

  const pushProblemReport = useCallback(() => {
    history.push(RoutePath.problemReport);
  }, [history]);

  return pushProblemReport;
};
