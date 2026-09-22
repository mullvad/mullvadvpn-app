import React from 'react';
import { useHistory } from 'react-router';

import { RoutePath } from '../../../../../../../../../../../shared/routes';
import { useTextFieldItemContext } from '../../../TextFieldItemContext';

export function useHandleFilterButtonClick() {
  const { id } = useTextFieldItemContext();
  const history = useHistory();

  const handleFilterButtonClick = React.useCallback(() => {
    const variant = id === 'entryAutomatic' ? 'entry' : id;

    history.push(RoutePath.filter, {
      options: [
        {
          type: 'filter-view-location-type',
          variant,
        },
      ],
    });
  }, [history, id]);

  return handleFilterButtonClick;
}
