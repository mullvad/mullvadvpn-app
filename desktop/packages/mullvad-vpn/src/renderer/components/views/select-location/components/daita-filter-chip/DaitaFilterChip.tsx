import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';

export type DaitaFilterChipProps = FilterChipProps;

export function DaitaFilterChip(props: DaitaFilterChipProps) {
  return (
    <FilterChip as="div" {...props}>
      <FilterChip.Text>
        {sprintf(messages.pgettext('select-location-view', 'Setting: %(settingName)s'), {
          settingName: 'DAITA',
        })}
      </FilterChip.Text>
    </FilterChip>
  );
}
