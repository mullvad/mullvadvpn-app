import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useDns, useSetDnsOption } from '../../hooks';

export type BlockGamblingSwitchProps = SwitchProps;

function BlockGamblingSwitch({ children, ...props }: BlockGamblingSwitchProps) {
  const { dns } = useDns();
  const setBlockGambling = useSetDnsOption('blockGambling');

  const disabled = dns.state === 'custom';
  const checked = dns.state === 'default' && dns.defaultOptions.blockGambling;

  return (
    <Switch disabled={disabled} checked={checked} onCheckedChange={setBlockGambling} {...props}>
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
