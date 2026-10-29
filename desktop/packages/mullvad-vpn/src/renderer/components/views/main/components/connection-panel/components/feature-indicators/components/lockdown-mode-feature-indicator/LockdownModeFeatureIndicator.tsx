import React from 'react';

import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function LockdownModeFeatureIndicator() {
  const history = useHistory();

  const gotoLockdownModeFeature = React.useCallback(() => {
    history.push(RoutePath.vpnSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'lockdown-mode-setting',
        },
      ],
    });
  }, [history]);

  return (
    <FeatureIndicator data-testid="feature-indicator" onClick={gotoLockdownModeFeature}>
      <FeatureIndicator.Text>
        {messages.pgettext('vpn-settings-view', 'Lockdown mode')}
      </FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
