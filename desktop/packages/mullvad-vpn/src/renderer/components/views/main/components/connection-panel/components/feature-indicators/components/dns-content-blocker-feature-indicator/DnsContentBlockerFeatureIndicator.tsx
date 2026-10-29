import React from 'react';

import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function DnsContentBlockersFeatureIndicator() {
  const history = useHistory();

  const gotoDnsContentBlockersFeature = React.useCallback(() => {
    history.push(RoutePath.vpnSettings, {
      transition: TransitionType.show,
      expandedSections: {
        'dns-blocker-setting': true,
      },
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'dns-blocker-setting',
        },
      ],
    });
  }, [history]);

  return (
    <FeatureIndicator data-testid="feature-indicator" onClick={gotoDnsContentBlockersFeature}>
      <FeatureIndicator.Text>
        {messages.pgettext('vpn-settings-view', 'DNS content blockers')}
      </FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
