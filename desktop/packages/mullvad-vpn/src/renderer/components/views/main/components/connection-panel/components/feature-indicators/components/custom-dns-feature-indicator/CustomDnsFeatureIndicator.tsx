import React from 'react';

import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function CustomDnsFeatureIndicator() {
  const history = useHistory();

  const gotoCustomDnsFeature = React.useCallback(() => {
    history.push(RoutePath.vpnSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'custom-dns-settings',
        },
      ],
    });
  }, [history]);

  return (
    <FeatureIndicator data-testid="feature-indicator" onClick={gotoCustomDnsFeature}>
      <FeatureIndicator.Text>
        {
          // TRANSLATORS: This refers to the Custom DNS setting in the VPN settings view. This is
          // TRANSLATORS: displayed when the feature is on.
          messages.gettext('Custom DNS')
        }
      </FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
