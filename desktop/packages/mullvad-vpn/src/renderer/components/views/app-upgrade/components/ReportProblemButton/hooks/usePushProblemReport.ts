import { useCallback } from 'react';
import { useHistory } from 'react-router';

import { RoutePath } from '../../../../../../lib/routes';

// TODO: Move this to lib/history/hooks when its PR has merged
export const usePushProblemReport = () => {
  const history = useHistory();

  const pushProblemReport = useCallback(() => {
    history.push(RoutePath.problemReport);
  }, [history]);

  return pushProblemReport;
};
