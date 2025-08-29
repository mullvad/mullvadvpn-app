import { useContext, useMemo } from 'react';

import { messages } from '../../../../shared/gettext';
import { IconButton } from '../../../lib/components';
import { TransitionType, useHistory } from '../../../lib/history';
import { BackActionContext } from '../../KeyboardNavigation';

export const AppNavigationHeaderBackButton = () => {
  const history = useHistory();
  // Compare the transition name with dismiss to infer wheter or not the view will slide
  // horizontally or vertically and then use matching button.
  const backIcon = useMemo(() => history.getPopTransition() !== TransitionType.dismiss, [history]);
  const { parentBackAction } = useContext(BackActionContext);

  if (!parentBackAction) return null;

  const iconSource = backIcon ? 'chevron-left-circle' : 'chevron-down-circle';
  const ariaLabel = backIcon ? messages.gettext('Back') : messages.gettext('Close');

  return (
    <IconButton variant="secondary" aria-label={ariaLabel} onClick={parentBackAction}>
      <IconButton.Icon icon={iconSource} />
    </IconButton>
  );
};
