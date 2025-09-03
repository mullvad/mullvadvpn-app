import { useWarningColor } from '../../hooks';
import { StyledCellWarningIcon } from './WarningIconStyles';

export function WarningIcon() {
  const warningColor = useWarningColor();

  return <StyledCellWarningIcon icon="alert-circle" color={warningColor} />;
}
