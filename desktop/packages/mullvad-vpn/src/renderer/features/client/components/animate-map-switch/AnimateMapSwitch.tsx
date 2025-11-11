import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useAnimateMap } from '../../hooks';

export type AnimateMapSwitchProps = SwitchProps;

function AnimateMapSwitch({ children, ...props }: AnimateMapSwitchProps) {
  const { animateMap, setAnimateMap } = useAnimateMap();

  return (
    <Switch checked={animateMap} onCheckedChange={setAnimateMap} {...props}>
      {children}
    </Switch>
  );
}

const AnimateMapSwitchNamespace = Object.assign(AnimateMapSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { AnimateMapSwitchNamespace as AnimateMapSwitch };
