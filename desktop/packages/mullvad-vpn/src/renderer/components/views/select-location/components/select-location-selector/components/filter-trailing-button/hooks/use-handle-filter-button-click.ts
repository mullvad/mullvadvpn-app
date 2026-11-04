import React from 'react';
import { useHistory } from 'react-router';

import { RoutePath } from '../../../../../../../../../shared/routes';
import useActions from '../../../../../../../../lib/actionsHook';
import userInterface from '../../../../../../../../redux/userinterface/actions';
import { useFilterTrailingButtonContext } from '../FilterTrailingButtonContext';

export function useHandleFilterButtonClick() {
  const { setSelectLocationView } = useActions(userInterface);

  const { type } = useFilterTrailingButtonContext();
  const history = useHistory();

  const handleFilterButtonClick = React.useCallback(() => {
    const locationType = type === 'automaticEntry' ? 'entry' : type;

    setSelectLocationView(locationType);

    history.push(RoutePath.filter, {
      options: [
        {
          type: 'filter-view-location-type',
          locationType,
        },
      ],
    });
  }, [history, setSelectLocationView, type]);

  return handleFilterButtonClick;
}
