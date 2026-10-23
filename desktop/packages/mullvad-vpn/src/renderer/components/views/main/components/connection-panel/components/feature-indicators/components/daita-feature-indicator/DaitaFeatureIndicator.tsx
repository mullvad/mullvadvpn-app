import React from 'react';

import { strings } from '../../../../../../../../../../shared/constants';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function DaitaFeatureIndicator() {
  const history = useHistory();

  const gotoEnableDaitaFeature = React.useCallback(() => {
    history.push(RoutePath.daitaSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'daita-enable-setting',
        },
      ],
    });
  }, [history]);

  return (
    <FeatureIndicator data-testid="feature-indicator" onClick={gotoEnableDaitaFeature}>
      <FeatureIndicator.Text>{strings.daita}</FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
