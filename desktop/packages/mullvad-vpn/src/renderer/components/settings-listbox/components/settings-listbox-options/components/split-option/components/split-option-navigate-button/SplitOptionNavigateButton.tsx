import { useCallback } from 'react';

import { RoutePath } from '../../../../../../../../../shared/routes';
import { ListItem } from '../../../../../../../../lib/components/list-item';
import { useHistory } from '../../../../../../../../lib/history';

export type NavigationOptionNavigateProps = {
  to: RoutePath;
} & React.ComponentPropsWithRef<'button'>;

export function SplitOptionNavigateButton({
  to,
  children,
  ...props
}: NavigationOptionNavigateProps) {
  const history = useHistory();
  const navigate = useCallback(() => {
    return history.push(to);
  }, [history, to]);

  return (
    <ListItem.TrailingActions>
      <ListItem.Trigger data-split-button onClick={navigate} tabIndex={-1} {...props}>
        <ListItem.TrailingActions.Action>
          <ListItem.TrailingActions.Action.Icon icon={'chevron-right'} aria-hidden="true" />
        </ListItem.TrailingActions.Action>
      </ListItem.Trigger>
    </ListItem.TrailingActions>
  );
}
