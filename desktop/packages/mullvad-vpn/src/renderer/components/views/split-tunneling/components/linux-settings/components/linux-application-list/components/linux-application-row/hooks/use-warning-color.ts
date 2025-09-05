import { Colors } from '../../../../../../../../../../lib/foundations';
import { useDisabled } from './use-disabled';

export function useWarningColor(): Colors {
  const disabled = useDisabled();

  const warningColor = disabled ? 'red' : 'yellow';

  return warningColor;
}
