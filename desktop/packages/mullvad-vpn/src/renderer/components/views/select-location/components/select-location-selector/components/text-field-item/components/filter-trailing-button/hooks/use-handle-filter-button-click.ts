import React from 'react';
import { useHistory } from 'react-router';

import { RoutePath } from '../../../../../../../../../../../shared/routes';
import { useTextFieldItemContext } from '../../../TextFieldItemContext';

export function useHandleFilterButtonClick() {
  const { id } = useTextFieldItemContext();
  const history = useHistory();

  const handleFilterButtonClick = React.useCallback(() => {
    const locationType = id === 'entryAutomatic' ? 'entry' : id;

    history.push(RoutePath.filter, {
      options: [
        {
          type: 'filter-view-location-type',
          locationType,
        },
      ],
    });
  }, [history, id]);

  return handleFilterButtonClick;
}
