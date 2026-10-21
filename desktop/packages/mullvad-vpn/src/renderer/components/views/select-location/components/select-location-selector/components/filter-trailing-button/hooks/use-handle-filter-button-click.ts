import React from 'react';
import { useHistory } from 'react-router';

import { RoutePath } from '../../../../../../../../../shared/routes';
import { useFilterTrailingButtonContext } from '../FilterTrailingButtonContext';

export function useHandleFilterButtonClick() {
  const { type } = useFilterTrailingButtonContext();
  const history = useHistory();

  const handleFilterButtonClick = React.useCallback(() => {
    const locationType = type === 'entryAutomatic' ? 'entry' : type;

    history.push(RoutePath.filter, {
      options: [
        {
          type: 'filter-view-location-type',
          locationType,
        },
      ],
    });
  }, [history, type]);

  return handleFilterButtonClick;
}
