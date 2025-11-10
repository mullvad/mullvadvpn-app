import { useDns } from '../../../../../../../features/dns/hooks';
import { Switch, SwitchProps } from '../../../../../../../lib/components/switch';

export type BlockAdsSwitchProps = SwitchProps;

function BlockAdsSwitch({ children, ...props }: BlockAdsSwitchProps) {
  const [dns, setBlockAds] = useDns('blockAds');

  return (
    <Switch
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockAds}
      onCheckedChange={setBlockAds}
      {...props}>
      {children}
    </Switch>
  );
}

const BlockAdsSwitchNamespace = Object.assign(BlockAdsSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { BlockAdsSwitchNamespace as BlockAdsSwitch };
