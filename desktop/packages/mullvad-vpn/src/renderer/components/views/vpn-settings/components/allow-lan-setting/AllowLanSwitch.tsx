import { useAppContext } from '../../../../../context';
import { useAllowLan } from '../../../../../features/lan-sharing/hooks';
import { Switch, SwitchProps } from '../../../../../lib/components/switch';

export type AllowLanSwitch = SwitchProps;

function AllowLanSwitch({ children, ...props }: AllowLanSwitch) {
  const allowLan = useAllowLan();
  const { setAllowLan } = useAppContext();

  return (
    <>
      <Switch checked={allowLan} onCheckedChange={setAllowLan} {...props}>
        {children}
      </Switch>
    </>
  );
}

const AllowLanSwitchNamespace = Object.assign(AllowLanSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { AllowLanSwitchNamespace as AllowLanSwitch };
