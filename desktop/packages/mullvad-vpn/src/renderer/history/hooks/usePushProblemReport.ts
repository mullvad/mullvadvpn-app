import { useCallback } from 'react';
import { useHistory } from 'react-router';

import { LocationState } from '../../../shared/ipc-types';
import { RoutePath } from '../../lib/routes';

export type PushProblemReportProps = {
  state?: Partial<LocationState>;
};

export const usePushProblemReport = ({ state }: PushProblemReportProps = {}) => {
  const history = useHistory();

  const pushProblemReport = useCallback(() => {
    history.push(
      {
        pathname: RoutePath.problemReport,
      },
      state,
    );
  }, [history, state]);

  return pushProblemReport;
};
