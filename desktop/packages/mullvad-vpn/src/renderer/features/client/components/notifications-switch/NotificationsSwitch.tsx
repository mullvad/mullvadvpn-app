import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useEnableSystemNotifications } from '../../hooks';

export type NotificationSwitchProps = SwitchProps;

function NotificationsSwitch({ children, ...props }: NotificationSwitchProps) {
  const { enableSystemNotifications, setEnableSystemNotifications } =
    useEnableSystemNotifications();

  return (
    <Switch
      checked={enableSystemNotifications}
      onCheckedChange={setEnableSystemNotifications}
      {...props}>
      {children}
    </Switch>
  );
}

const NotificationSwitchNamespace = Object.assign(NotificationsSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { NotificationSwitchNamespace as NotificationsSwitch };
