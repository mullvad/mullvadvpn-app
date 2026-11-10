import React from 'react';

import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function ShadowsocksFeatureIndicator() {
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
    <FeatureIndicator data-testid="feature-indicator" onClick={gotoAntiCensorship}>
      <FeatureIndicator.Text>
        {messages.pgettext('wireguard-settings-view', 'Shadowsocks')}
      </FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
