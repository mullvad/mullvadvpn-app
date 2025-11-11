import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useQuantumResistant() {
  const quantumResistant = useSelector((state) => state.settings.wireguard.quantumResistant);
  const { setWireguardQuantumResistant } = useAppContext();
  return { quantumResistant, setWireguardQuantumResistant };
}
