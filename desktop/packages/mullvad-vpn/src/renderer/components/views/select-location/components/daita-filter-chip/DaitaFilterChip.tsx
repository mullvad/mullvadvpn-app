import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../lib/history';

export type DaitaFilterChipProps = FilterChipProps;

export function DaitaFilterChip(props: DaitaFilterChipProps) {
  const history = useHistory();
  const gotoEnableDaitaFeature = React.useCallback(() => {
    history.push(RoutePath.daitaSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'daita-enable-setting',
        },
      ],
    });
  }, [history]);

  return (
    <FilterChip
      as="a"
      aria-label={
        // TRANSLATORS: Accessibility label for the button that navigates to the DAITA settings.
        messages.pgettext('accessibility', 'Go to DAITA settings')
      }
      onClick={gotoEnableDaitaFeature}
      {...props}>
      <FilterChip.Text>
        {sprintf(messages.pgettext('select-location-view', 'Setting: %(settingName)s'), {
          settingName: 'DAITA',
        })}
      </FilterChip.Text>
    </FilterChip>
  );
}
