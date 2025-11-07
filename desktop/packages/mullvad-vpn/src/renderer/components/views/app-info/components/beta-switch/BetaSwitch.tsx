import { Switch, SwitchProps } from '../../../../../lib/components/switch';
import { useSettingsShowBetaReleases, useVersionIsBeta } from '../../../../../redux/hooks';

export type BetaSwitchProps = SwitchProps;

function BetaSwitch({ children, ...props }: BetaSwitchProps) {
  const { isBeta } = useVersionIsBeta();
  const { showBetaReleases, setShowBetaReleases } = useSettingsShowBetaReleases();

  return (
    <Switch
      checked={showBetaReleases}
      onCheckedChange={setShowBetaReleases}
      disabled={isBeta}
      {...props}>
      {children}
    </Switch>
  );
}

const BetaSwitchNamespace = Object.assign(BetaSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { BetaSwitchNamespace as BetaSwitch };
