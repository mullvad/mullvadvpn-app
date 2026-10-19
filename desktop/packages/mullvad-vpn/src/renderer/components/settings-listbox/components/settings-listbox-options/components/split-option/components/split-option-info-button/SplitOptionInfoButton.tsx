import { ListItem } from '../../../../../../../../lib/components/list-item';

export type NavigationOptionInfoProps = React.ComponentPropsWithRef<'button'>;

export function SplitOptionInfoButton({ children, ...props }: NavigationOptionInfoProps) {
  return (
    <ListItem.TrailingActions>
      <ListItem.Trigger data-split-button tabIndex={-1} {...props}>
        <ListItem.TrailingActions.Action>
          <ListItem.TrailingActions.Action.Icon icon="info-circle" aria-hidden="true" />
        </ListItem.TrailingActions.Action>
      </ListItem.Trigger>
    </ListItem.TrailingActions>
  );
}
