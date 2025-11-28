import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useQuantumResistant } from '../../hooks';

export type QuantumResistantSwitchProp = SwitchProps;

function QuantumResistantSwitch({ children, ...props }: QuantumResistantSwitchProp) {
  const { quantumResistant, setQuantumResistant } = useQuantumResistant();

  return (
    <Switch checked={quantumResistant} onCheckedChange={setQuantumResistant} {...props}>
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
