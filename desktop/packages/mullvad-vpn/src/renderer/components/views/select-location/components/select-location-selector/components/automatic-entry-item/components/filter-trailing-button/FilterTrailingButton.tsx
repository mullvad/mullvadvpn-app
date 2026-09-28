import React from 'react';

import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { LocationSelector } from '../../../../../../../../../lib/components/location-selector';
import { useHistory } from '../../../../../../../../../lib/history';
import { useIcon } from './hooks';

export function AutomaticEntryFilterTrailingButton() {
  const icon = useIcon();
  const history = useHistory();

  const handleClick = React.useCallback(() => {
    history.push(RoutePath.filter, {
      options: [
        {
          type: 'filter-view-location-type',
          locationType: 'entry',
        },
      ],
    });
  }, [history]);

  return (
    <LocationSelector.Items.TextFieldItem.TrailingButton
      aria-label={
        // TRANSLATORS: Label provided to accessibility tools such as screenreaders to inform the user
        // TRANSLATORS: that entry location filters are disabled due to the use of the Automatic entry location.
        messages.pgettext(
          'accessibility',
          'Entry location filters are overridden due to use of Automatic entry location',
        )
      }
      visible={true}
      onClick={handleClick}>
      <LocationSelector.Items.TextFieldItem.TrailingButton.Icon icon={icon} />
    </LocationSelector.Items.TextFieldItem.TrailingButton>
  );
}
