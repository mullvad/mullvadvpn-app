import React from 'react';

import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function LanSharingFeatureIndicator() {
  const history = useHistory();

  const gotoLanSharingFeature = React.useCallback(() => {
    history.push(RoutePath.vpnSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'allow-lan-setting',
        },
      ],
    });
  }, [history]);

  return (
    <FeatureIndicator data-testid="feature-indicator" onClick={gotoLanSharingFeature}>
      <FeatureIndicator.Text>
        {messages.pgettext('vpn-settings-view', 'Local network sharing')}
      </FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
