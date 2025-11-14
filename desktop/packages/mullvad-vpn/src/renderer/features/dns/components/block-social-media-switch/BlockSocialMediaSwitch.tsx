import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useDns, useSetDnsOption } from '../../hooks';

export type BlockSocialMediaSwitchProps = SwitchProps;

function BlockSocialMediaSwitch({ children, ...props }: BlockSocialMediaSwitchProps) {
  const { dns } = useDns();
  const setBlockSocialMedia = useSetDnsOption('blockSocialMedia');

  return (
    <Switch
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockSocialMedia}
      onCheckedChange={setBlockSocialMedia}
      {...props}>
      {children}
    </Switch>
  );
}

const BlockSocialMediaSwitchNamespace = Object.assign(BlockSocialMediaSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { BlockSocialMediaSwitchNamespace as BlockSocialMediaSwitch };
