import { useSelector } from '../../../redux/store';

export function useQuantumResistant() {
  const quantumResistant = useSelector((state) => state.settings.wireguard.quantumResistant);
  return quantumResistant;
}
