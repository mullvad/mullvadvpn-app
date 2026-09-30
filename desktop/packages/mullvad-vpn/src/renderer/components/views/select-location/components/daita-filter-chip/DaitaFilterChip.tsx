import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { LocationType } from '../../../../../features/locations/types';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../lib/history';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

export type DaitaFilterChipProps = FilterChipProps;

export function DaitaFilterChip(props: DaitaFilterChipProps) {
  const history = useHistory();

  const { locationType } = useSelectLocationViewContext();
  const inactive = locationType === LocationType.entryAutomatic;

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
      tabIndex={0}
      aria-label={
        // TRANSLATORS: Accessibility label for link to DAITA settings.
        messages.pgettext('accessibility', 'DAITA settings')
      }
      inactive={inactive}
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
