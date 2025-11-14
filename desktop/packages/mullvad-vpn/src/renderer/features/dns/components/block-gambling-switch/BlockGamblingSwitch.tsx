import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useDns, useSetDnsOption } from '../../hooks';

export type BlockGamblingSwitchProps = SwitchProps;

function BlockGamblingSwitch({ children, ...props }: BlockGamblingSwitchProps) {
  const { dns } = useDns();
  const setBlockGambling = useSetDnsOption('blockGambling');

  return (
    <Switch
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockGambling}
      onCheckedChange={setBlockGambling}
      {...props}>
      {children}
    </Switch>
  );
}

const BlockGamblingSwitchNamespace = Object.assign(BlockGamblingSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { BlockGamblingSwitchNamespace as BlockGamblingSwitch };
