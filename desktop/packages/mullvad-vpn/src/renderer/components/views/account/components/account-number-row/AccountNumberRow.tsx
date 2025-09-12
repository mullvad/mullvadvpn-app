import { useSelector } from '../../../../../redux/store';
import AccountNumberLabel from '../../../../AccountNumberLabel';
import { AccountRowValue } from '../../AccountStyles';

export function AccountNumberRow() {
  const accountNumber = useSelector((state) => state.account.accountNumber);
  return <AccountRowValue as={AccountNumberLabel} accountNumber={accountNumber || ''} />;
}
