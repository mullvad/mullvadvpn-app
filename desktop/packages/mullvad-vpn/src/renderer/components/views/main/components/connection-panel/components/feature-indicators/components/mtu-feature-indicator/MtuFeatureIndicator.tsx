import React from 'react';

import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function MtuFeatureIndicator() {
  const history = useHistory();

  const gotoMtuFeature = React.useCallback(() => {
    history.push(RoutePath.vpnSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'mtu-setting',
        },
      ],
    });
  }, [history]);

  return (
    <FeatureIndicator data-testid="feature-indicator" onClick={gotoMtuFeature}>
      <FeatureIndicator.Text>
        {messages.pgettext('wireguard-settings-view', 'MTU')}
      </FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
