import React from 'react';

import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function MagicMultihopFeatureIndicator() {
  const history = useHistory();

  const gotoMultihopFeature = React.useCallback(() => {
    history.push(RoutePath.multihopSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'multihop-setting',
        },
      ],
    });
  }, [history]);

  return (
    <FeatureIndicator data-testid="feature-indicator" onClick={gotoMultihopFeature}>
      <FeatureIndicator.Text>
        {
          // TRANSLATORS: This is used as a feature indicator to show that multihop is used
          // TRANSLATORS: to enable the connection.
          messages.pgettext('connect-view', 'Automatic multihop')
        }
      </FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
