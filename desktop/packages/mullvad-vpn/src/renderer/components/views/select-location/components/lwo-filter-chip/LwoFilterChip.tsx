import React from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { FilterChip, type FilterChipProps } from '../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../lib/history';

export type LwoFilterChipProps = FilterChipProps;

export function LwoFilterChip(props: LwoFilterChipProps) {
  const history = useHistory();

  const gotoAntiCensorship = React.useCallback(() => {
    history.push(RoutePath.antiCensorship, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'obfuscation-setting',
        },
      ],
    });
  }, [history]);

  return (
    <FilterChip
      as="a"
      aria-label={
        // TRANSLATORS: Accessibility label for the button that navigates to the anti-censorship settings.
        messages.pgettext('accessibility', 'Go to anti-censorship settings')
      }
      onClick={gotoAntiCensorship}
      {...props}>
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
