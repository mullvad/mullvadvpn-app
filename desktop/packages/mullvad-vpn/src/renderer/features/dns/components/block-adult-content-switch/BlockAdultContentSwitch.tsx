import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useDns, useSetDnsOption } from '../../hooks';

export type BlockAdultContentSwitchProps = SwitchProps;

function BlockAdultContentSwitch({ children, ...props }: BlockAdultContentSwitchProps) {
  const { dns } = useDns();
  const setBlockAdultContent = useSetDnsOption('blockAdultContent');

  const disabled = dns.state === 'custom';
  const checked = dns.state === 'default' && dns.defaultOptions.blockAdultContent;

  return (
    <Switch disabled={disabled} checked={checked} onCheckedChange={setBlockAdultContent} {...props}>
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
