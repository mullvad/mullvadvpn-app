import React from 'react';

import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { FeatureIndicator } from '../../../../../../../../../lib/components';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export function MagicMultihopFeatureIndicator() {
  const history = useHistory();

  const gotoMultihopFeature = React.useCallback(() => {
    history.push(RoutePath.multihopSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'multihop-setting',
        },
      ],
    });
  }, [history]);

  return (
    <FeatureIndicator data-testid="feature-indicator" onClick={gotoMultihopFeature}>
      <FeatureIndicator.Icon
        icon="magic-multihop"
        aria-label={
          // TRANSLATORS: Accessible explanation for the symbol indicating automatic multihop.
          messages.pgettext('accessibility', 'Automatic multihop symbol')
        }
      />
      <FeatureIndicator.Text>{messages.gettext('Multihop')}</FeatureIndicator.Text>
    </FeatureIndicator>
  );
}
