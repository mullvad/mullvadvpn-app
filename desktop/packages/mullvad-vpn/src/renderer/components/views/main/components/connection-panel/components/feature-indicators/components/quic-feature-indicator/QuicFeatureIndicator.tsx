import React from 'react';

import { strings } from '../../../../../../../../../../shared/constants';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function QuicFeatureIndicator() {
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
      <FeatureIndicator.Text>{strings.quic}</FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
