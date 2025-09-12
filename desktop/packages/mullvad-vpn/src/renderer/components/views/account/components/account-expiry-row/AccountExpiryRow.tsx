import { useSelector } from '../../../../../redux/store';
import { FormattedAccountExpiry } from '../formatted-account-expiry/FormattedAccountExpiry';

export function AccountExpiryRow() {
  const accountExpiry = useSelector((state) => state.account.expiry);
  const expiryLocale = useSelector((state) => state.userInterface.locale);
  return <FormattedAccountExpiry expiry={accountExpiry} locale={expiryLocale} />;
}
