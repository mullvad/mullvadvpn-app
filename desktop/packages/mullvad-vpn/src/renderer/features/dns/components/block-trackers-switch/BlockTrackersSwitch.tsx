import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useDns, useSetDnsOption } from '../../hooks';

export type BlockTrackersSwitchProps = SwitchProps;

function BlockTrackersSwitch({ children, ...props }: BlockTrackersSwitchProps) {
  const { dns } = useDns();
  const setBlockTrackers = useSetDnsOption('blockTrackers');

  const disabled = dns.state === 'custom';
  const checked = dns.state === 'default' && dns.defaultOptions.blockTrackers;

  return (
    <Switch disabled={disabled} checked={checked} onCheckedChange={setBlockTrackers} {...props}>
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
