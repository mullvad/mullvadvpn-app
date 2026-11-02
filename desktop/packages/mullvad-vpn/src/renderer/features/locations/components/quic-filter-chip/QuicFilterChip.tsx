import React from 'react';

import { strings } from '../../../../../shared/constants';
import { messages } from '../../../../../shared/gettext';
import { RoutePath } from '../../../../../shared/routes';
import { FilterChip, type FilterChipProps } from '../../../../lib/components';
import { TransitionType, useHistory } from '../../../../lib/history';

export type QuicFilterChipProps = FilterChipProps;

export function QuicFilterChip(props: QuicFilterChipProps) {
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
      tabIndex={0}
      aria-label={
        // TRANSLATORS: Accessibility label for link to anti-censorship settings.
        messages.pgettext('accessibility', 'Anti-censorship settings')
      }
      onClick={gotoAntiCensorship}
      {...props}>
      <FilterChip.Text>{strings.quic}</FilterChip.Text>
    </FilterChip>
  );
}
