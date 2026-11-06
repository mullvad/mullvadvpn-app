import React from 'react';

import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function MultihopFeatureIndicator() {
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
          // TRANSLATORS: This refers to the multihop setting in the VPN settings view. This is
          // TRANSLATORS: displayed when the feature is on.
          messages.gettext('Multihop')
        }
      </FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
