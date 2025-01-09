import { useCallback } from 'react';

import { messages } from '../../../../../shared/gettext';
import { useHistory } from '../../../../lib/history';
import { RoutePath } from '../../../../lib/routes';
import * as Cell from '../../../cell';

export function ChangelogListItem() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.changelog), [history]);

  return (
    <Cell.CellNavigationButton onClick={navigate}>
      <Cell.Label>{messages.pgettext('settings-view', 'What’s new')}</Cell.Label>
    </Cell.CellNavigationButton>
  );
}
