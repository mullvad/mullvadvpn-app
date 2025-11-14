import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useDns, useSetDnsOption } from '../../hooks';

export type BlockAdultContentSwitchProps = SwitchProps;

function BlockAdultContentSwitch({ children, ...props }: BlockAdultContentSwitchProps) {
  const { dns } = useDns();
  const setBlockAdultContent = useSetDnsOption('blockAdultContent');

  return (
    <Switch
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockAdultContent}
      onCheckedChange={setBlockAdultContent}
      {...props}>
      {children}
    </Switch>
  );
}

const BlockAdultContentSwitchNamespace = Object.assign(BlockAdultContentSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { BlockAdultContentSwitchNamespace as BlockAdultContentSwitch };
