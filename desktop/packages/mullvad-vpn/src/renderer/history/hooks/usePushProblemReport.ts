import { useCallback } from 'react';
import { useHistory } from 'react-router';

import { RoutePath } from '../../lib/routes';

export type PushProblemReportProps = {
  search?: string;
};

export const usePushProblemReport = ({ search }: PushProblemReportProps = {}) => {
  const history = useHistory();

  const pushProblemReport = useCallback(() => {
    history.push({
      pathname: RoutePath.problemReport,
      search,
    });
  }, [history, search]);

  return pushProblemReport;
};
