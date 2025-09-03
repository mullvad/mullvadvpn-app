import { StyledCellLabel } from './ApplicationLabelStyles';

export type ApplicationLabelProps = {
  children: React.ReactNode;
  disabled?: boolean;
};

export function ApplicationLabel({ children, disabled }: ApplicationLabelProps) {
  return <StyledCellLabel $lookDisabled={disabled}>{children}</StyledCellLabel>;
}
