import { useAppContext } from '../../../../../context';
import { useQuantumResistant } from '../../../../../features/tunnel/hooks';
import { Switch, SwitchProps } from '../../../../../lib/components/switch';

export type QuantumResistantSwitchProp = SwitchProps;

function QuantumResistantSwitch({ children, ...props }: QuantumResistantSwitchProp) {
  const { setWireguardQuantumResistant } = useAppContext();
  const quantumResistant = useQuantumResistant();

  return (
    <Switch checked={quantumResistant} onCheckedChange={setWireguardQuantumResistant} {...props}>
      {children}
    </Switch>
  );
}

const QuantumResistantSwitchNamespace = Object.assign(QuantumResistantSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { QuantumResistantSwitchNamespace as QuantumResistantSwitch };
