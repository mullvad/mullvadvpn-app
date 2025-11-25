import { useUserInterfaceConnectedToDaemon } from '../../../../../../../../../../redux/hooks';
import { useHasUpgrade } from '../../../../../../../hooks';

export function useDisabled() {
  const { connectedToDaemon } = useUserInterfaceConnectedToDaemon();
  const hasUpgrade = useHasUpgrade();

  const disabled = !hasUpgrade || !connectedToDaemon;

  return disabled;
}
