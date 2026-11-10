import React from 'react';

import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function ServerIpOverrideFeatureIndicator() {
  const history = useHistory();

  const gotoServerIpOverride = React.useCallback(() => {
    history.push(RoutePath.settingsImport, {
      transition: TransitionType.show,
    });
  }, [history]);

  return (
    <FeatureIndicator data-testid="feature-indicator" onClick={gotoServerIpOverride}>
      <FeatureIndicator.Text>
        {messages.pgettext('settings-import', 'Server IP override')}
      </FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
