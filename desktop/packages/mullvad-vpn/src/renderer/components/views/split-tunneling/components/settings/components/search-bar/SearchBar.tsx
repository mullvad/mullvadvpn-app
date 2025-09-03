import { StyledSearchBar } from '../../../../SplitTunnelingStyles';
import { useSettingsContext } from '../../SettingsContext';

export function SearchBar() {
  const { searchTerm, setSearchTerm } = useSettingsContext();

  return <StyledSearchBar searchTerm={searchTerm} onSearch={setSearchTerm} />;
}
