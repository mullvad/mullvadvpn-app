import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { useDaitaEnabled } from '../../hooks';

export type DaitaSwitchProps = SwitchProps;

function DaitaSwitch({ children, ...props }: DaitaSwitchProps) {
  const { daitaEnabled, setDaitaEnabled } = useDaitaEnabled();

  const relaySettings = useNormalRelaySettings();
  const disabled = relaySettings === undefined;

  return (
    <Switch checked={daitaEnabled} onCheckedChange={setDaitaEnabled} disabled={disabled} {...props}>
      {children}
    </Switch>
  );
}

const DaitaSwitchNamespace = Object.assign(DaitaSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { DaitaSwitchNamespace as DaitaSwitch };
