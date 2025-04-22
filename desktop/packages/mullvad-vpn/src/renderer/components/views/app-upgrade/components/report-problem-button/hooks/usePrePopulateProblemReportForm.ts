import { useCallback } from 'react';

import useActions from '../../../../../../lib/actionsHook';
import supportActions from '../../../../../../redux/support/actions';
import { useMessage } from './useMessage';

export const usePrePopulateProblemReportForm = () => {
  const { saveReportForm } = useActions(supportActions);
  const message = useMessage();

  const prePopulateProblemReportForm = useCallback(() => {
    if (message) {
      saveReportForm({
        email: '',
        message,
      });
    }
  }, [message, saveReportForm]);

  return prePopulateProblemReportForm;
};
