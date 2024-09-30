import { formatAccountNumber } from '../lib/account';
import ClipboardLabel from './ClipboardLabel';

interface IAccountNumberLabelProps {
  accountNumber: string;
  obscureValue?: boolean;
  className?: string;
}

export default function AccountNumberLabel(props: IAccountNumberLabelProps) {
  return (
    <ClipboardLabel
      value={props.accountNumber}
      displayValue={formatAccountNumber(props.accountNumber)}
      obscureValue={props.obscureValue}
      className={props.className}
      data-testid="account-number"
    />
  );
}
