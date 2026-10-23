import React from 'react';

import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function QuantumResistanceFeatureIndicator() {
  const history = useHistory();

  const gotoQuantumResistantFeature = React.useCallback(() => {
    history.push(RoutePath.vpnSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'quantum-resistant-setting',
        },
      ],
    });
  }, [history]);

  return (
    <FeatureIndicator data-testid="feature-indicator" onClick={gotoQuantumResistantFeature}>
      <FeatureIndicator.Text>{messages.gettext('Quantum resistance')}</FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
