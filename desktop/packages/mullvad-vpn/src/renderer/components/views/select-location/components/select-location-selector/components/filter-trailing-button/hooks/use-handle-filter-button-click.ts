import React from 'react';
import { useHistory } from 'react-router';

import { RoutePath } from '../../../../../../../../../shared/routes';
import { LocationType } from '../../../../../../../../features/locations/types';
import useActions from '../../../../../../../../lib/actionsHook';
import userInterface from '../../../../../../../../redux/userinterface/actions';
import { useFilterTrailingButtonContext } from '../FilterTrailingButtonContext';

export function useHandleFilterButtonClick() {
  const { setSelectLocationView } = useActions(userInterface);

  const { type } = useFilterTrailingButtonContext();
  const history = useHistory();

  // TODO: Remove this conversion logic when `LocationType` has been replaced with
  // a regular union type
  const locationTypeAsEnum = (locationType: 'entry' | 'exit' | 'entryAutomatic') => {
    switch (locationType) {
      case 'entry':
        return LocationType.entry;
      case 'entryAutomatic':
        return LocationType.entryAutomatic;
      case 'exit':
        return LocationType.exit;
      default:
        return locationType satisfies never;
    }
  };

  const handleFilterButtonClick = React.useCallback(() => {
    const locationType = type === 'entryAutomatic' ? 'entry' : type;

    const unregisterListener = history.listen(async (location) => {
      if (location.pathname === RoutePath.filter) {
        await document.activeViewTransition?.finished;
        setSelectLocationView(locationTypeAsEnum(type));
      }
    });

    history.push(RoutePath.filter, {
      options: [
        {
          type: 'filter-view-location-type',
          locationType,
        },
      ],
    });

    return () => {
      unregisterListener();
    };
  }, [history, setSelectLocationView, type]);

  return handleFilterButtonClick;
}
