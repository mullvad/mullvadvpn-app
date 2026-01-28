import { useCallback } from 'react';

import { RoutePath } from '../../../../../../../shared/routes';
import { ListItem } from '../../../../../../lib/components/list-item';
import { useHistory } from '../../../../../../lib/history';

export type NavigationOptionNavigateProps = {
  to: RoutePath;
} & React.ComponentPropsWithRef<'div'>;

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
    <ListItem.Trigger data-split-button onClick={navigate} tabIndex={-1} {...props}>
      <ListItem.TrailingAction>
        <ListItem.TrailingAction.Icon icon={'chevron-right'} aria-hidden="true" />
      </ListItem.TrailingAction>
    </ListItem.Trigger>
  );
}
