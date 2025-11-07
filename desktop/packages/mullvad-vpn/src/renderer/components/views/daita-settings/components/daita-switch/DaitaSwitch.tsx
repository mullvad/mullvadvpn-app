import { useAppContext } from '../../../../../context';
import { useDaitaEnabled } from '../../../../../features/daita/hooks';
import { Switch, SwitchProps } from '../../../../../lib/components/switch';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';

export type DaitaSwitchProps = SwitchProps;

function DaitaSwitch({ children, ...props }: DaitaSwitchProps) {
  const { setEnableDaita } = useAppContext();
  const checked = useDaitaEnabled();

  const relaySettings = useNormalRelaySettings();
  const disabled = relaySettings === undefined;

  return (
    <Switch checked={checked} onCheckedChange={setEnableDaita} disabled={disabled} {...props}>
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
