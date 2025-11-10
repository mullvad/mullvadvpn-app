import { useDns } from '../../../../../../../features/dns/hooks';
import { Switch, SwitchProps } from '../../../../../../../lib/components/switch';

export type BlockTrackersSwitchProps = SwitchProps;

function BlockTrackersSwitch({ children, ...props }: BlockTrackersSwitchProps) {
  const [dns, setBlockTrackers] = useDns('blockTrackers');

  return (
    <Switch
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockTrackers}
      onCheckedChange={setBlockTrackers}
      {...props}>
      {children}
    </Switch>
  );
}

const BlockTrackersSwitchNamespace = Object.assign(BlockTrackersSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { BlockTrackersSwitchNamespace as BlockTrackersSwitch };
