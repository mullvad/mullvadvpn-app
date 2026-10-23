import React from 'react';

import { strings } from '../../../../../../../../../../shared/constants';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function SplitTunnelingFeatureIndicator() {
  const history = useHistory();

  const gotoSplitTunnelingFeature = React.useCallback(() => {
    history.push(RoutePath.splitTunneling, {
      transition: TransitionType.show,
    });
  }, [history]);

  return (
    <FeatureIndicator data-testid="feature-indicator" onClick={gotoSplitTunnelingFeature}>
      <FeatureIndicator.Text>{strings.splitTunneling}</FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
