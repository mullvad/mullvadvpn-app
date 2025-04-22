import { useCallback } from 'react';

import { usePushProblemReport } from '../../../../../../history/hooks';
import { usePrePopulateProblemReportForm } from './usePrePopulateProblemReportForm';

export const useHandleClick = () => {
  const prePopulateProblemReportForm = usePrePopulateProblemReportForm();
  const pushProblemReport = usePushProblemReport({
    state: {
      options: [{ type: 'suppress-outdated-version-warning' }],
    },
  });

  const handleClick = useCallback(() => {
    prePopulateProblemReportForm();
    pushProblemReport();
  }, [prePopulateProblemReportForm, pushProblemReport]);

  return handleClick;
};
