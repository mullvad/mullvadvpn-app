import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';

export type LwoFilterChipProps = FilterChipProps;

export function LwoFilterChip(props: LwoFilterChipProps) {
  return (
    <FilterChip as="div" {...props}>
      <FilterChip.Text>
        {sprintf(
          // TRANSLATORS: Label for indicator that shows that obfuscation is being used as a filter.
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: %(obfuscation)s - type of obfuscation in use
          messages.pgettext('select-location-view', 'Obfuscation: %(obfuscation)s'),
          { obfuscation: strings.lwo },
        )}
      </FilterChip.Text>
    </FilterChip>
  );
}
