import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useMonochromaticTrayIcon } from '../../hooks';

export type MonochromaticTrayIconSwitchProps = SwitchProps;

function MonochromaticTrayIconSwitch({ children, ...props }: MonochromaticTrayIconSwitchProps) {
  const { monochromaticIcon, setMonochromaticIcon } = useMonochromaticTrayIcon();

  return (
    <Switch checked={monochromaticIcon} onCheckedChange={setMonochromaticIcon} {...props}>
      {children}
    </Switch>
  );
}

const MonochromaticTrayIconSwitchNamespace = Object.assign(MonochromaticTrayIconSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { MonochromaticTrayIconSwitchNamespace as MonochromaticTrayIconSwitch };
